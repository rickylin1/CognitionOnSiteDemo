// LEGACY: read-only reference for porting. Not built or run in this repo.
// NOT ported to Rust yet.
package com.example.telemetry

import java.sql.Timestamp

import org.apache.spark.sql.{Dataset, SparkSession}
import org.apache.spark.sql.functions._
import org.apache.spark.sql.streaming.{GroupState, GroupStateTimeout, OutputMode}

// Event and the input schema are shared with BatteryHealthJob.scala.
case class OpenSession(start: Event)

case class ChargingSession(vin: String, session_start: Timestamp, session_end: Timestamp,
                           duration_min: Long, start_soc_pct: Int, end_soc_pct: Int,
                           soc_gained_pct: Int, lat: Double, lon: Double)

object ChargingSessionJob {
  private val log = org.apache.log4j.Logger.getLogger(getClass)

  // R4: location leaves the job only rounded to 2 decimal places.
  def round2(x: Double): Double = BigDecimal(x).setScale(2, BigDecimal.RoundingMode.HALF_UP).toDouble

  def sessions(vin: String, events: Iterator[Event],
               state: GroupState[OpenSession]): Iterator[ChargingSession] = {
    // R2: event-time order (ts, then event_id), never arrival order.
    val sorted = events.toSeq.sortBy(e => (e.ts.getTime, e.event_id))
    var open = state.getOption.map(_.start)
    val closed = scala.collection.mutable.ArrayBuffer.empty[ChargingSession]

    for (e <- sorted) e.event_type match {
      case "charge_start" if open.isEmpty => open = Some(e)   // opens a session
      case "charge_start"                 => ()               // already open: ignore
      case "charge_stop" if open.isDefined =>                 // closes the open session
        val s = open.get
        closed += ChargingSession(
          vin = vin,
          session_start = s.ts,
          session_end = e.ts,
          duration_min = (e.ts.getTime - s.ts.getTime) / 60000, // floored minutes
          start_soc_pct = s.soc_pct,
          end_soc_pct = e.soc_pct,
          soc_gained_pct = e.soc_pct - s.soc_pct,
          lat = round2(s.lat), // location comes from the charge_start
          lon = round2(s.lon))
        open = None
      case _ => ()                                            // orphan stop: ignore
    }

    // Sessions never closed are kept in state and never emitted.
    open match {
      case Some(s) => state.update(OpenSession(s))
      case None    => state.remove()
    }
    closed.iterator
  }

  def main(args: Array[String]): Unit = {
    val spark = SparkSession.builder.appName("charging_session").getOrCreate()
    import spark.implicits._

    val events: Dataset[Event] = spark.readStream
      .format("kafka")
      .option("kafka.bootstrap.servers", sys.env("KAFKA_BOOTSTRAP_SERVERS"))
      .option("subscribe", "vehicle-telemetry")
      .option("startingOffsets", "earliest")
      .load()
      .select(from_json($"value".cast("string"), BatteryHealthJob.schema).as("e"))
      .select("e.*")
      .as[Event]

    // R1: exactly-once. Deduplicate by event_id before anything else.
    val unique = events
      .withWatermark("ts", "24 hours")
      .dropDuplicatesWithinWatermark("event_id")

    val closed = unique
      .filter($"event_type".isin("charge_start", "charge_stop"))
      .groupByKey(_.vin)
      .flatMapGroupsWithState(OutputMode.Append(), GroupStateTimeout.NoTimeout)(sessions)

    closed.writeStream
      .outputMode(OutputMode.Append())
      .foreachBatch { (batch: Dataset[ChargingSession], batchId: Long) =>
        val out = batch.orderBy("vin", "session_start")
        out.write.mode("append").json(sys.env("CHARGING_SESSION_OUTPUT_PATH"))
        // R3: log counts only. Never VIN, lat or lon.
        log.info(s"charging_session batch=$batchId records written=${out.count()}")
      }
      .start()
      .awaitTermination()
  }
}
