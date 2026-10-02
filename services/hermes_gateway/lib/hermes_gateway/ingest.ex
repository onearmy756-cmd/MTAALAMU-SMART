defmodule HermesGateway.Ingest do
  @moduledoc """
  Pipeline ya telemetry: Store → Rules → Alerts → HERMES mission (`iot_incident`).

  Kila event inaandikwa kwenye telemetry ya hivi punde (ETS). Rule FAIL inatengeneza
  alert na kuanzisha mission ya HERMES (memory ya kudumu kwenye Store).
  """

  def ingest(device_id, fields, source, meta \\ %{}) when is_map(fields) do
    HermesGateway.Store.put_telemetry(device_id, fields, Map.put(meta, :source, source))
    res = HermesGateway.Rules.evaluate(device_id, fields)

    # HERMES mission kwa kila event yenye FAIL (incident pipeline)
    alerts = res["alerts"] || []
    mission_id =
      if length(alerts) > 0 do
        {:ok, mid, _rec} =
          HermesGateway.Store.new_mission("iot_incident", %{
            device: device_id,
            fields: fields,
            source: source,
            alerts: Enum.map(alerts, & &1.id)
          })

        HermesGateway.Store.add_frame(mid, %{
          agent: "iot_specialist",
          step: "ingest",
          ts: System.system_time(:second),
          narration_sw: "Nimepokea telemetry yenye hatari kutoka #{device_id}. Nitaanza utambuzi.",
          evidence: %{"fields" => fields, "source" => source}
        })

        mid
      else
        nil
      end

    Map.put(res, "mission_id", mission_id)
  end
end
