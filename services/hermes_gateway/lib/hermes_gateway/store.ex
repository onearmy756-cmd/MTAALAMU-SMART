defmodule HermesGateway.Store do
  @moduledoc """
  Memory ya HERMES (persistent pattern, ETS-backed):
  telemetry ya hivi punde, alerts zilizofunguliwa, missions + frames, HITL requests.

  Snapshot kwa audit: `Store.snapshot/0`. Chanzo cha kudumu: data/iot/*.json
  (missions/alerts zinaweza kuflushiwa kwa `Store.persist/1` — mtiririko wa JSON, KANUNI R-183).
  """
  use GenServer
  alias :ets, as: Ets

  @tables [:hermes_telemetry, :hermes_alerts, :hermes_missions, :hermes_hitl]

  @impl true
  def init(_arg) do
    Enum.each(@tables, fn t ->
      Ets.new(t, [:named_table, :set, :public, read_concurrency: true])
    end)
    {:ok, %{}}
  end

  def start_link(_arg), do: GenServer.start_link(__MODULE__, nil, name: __MODULE__)

  # --- telemetry -----------------------------------------------------------
  def put_telemetry(device_id, fields, meta) do
    Ets.insert(:hermes_telemetry, {device_id, %{fields: fields, meta: meta, ts: System.system_time(:second)}})
  end

  def get_telemetry(device_id), do: lookup(:hermes_telemetry, device_id)

  def all_telemetry do
    Ets.tab2list(:hermes_telemetry) |> Enum.map(fn {k, v} -> {k, v} end) |> Enum.into(%{})
  end

  # --- alerts ---------------------------------------------------------------
  def add_alert(%{} = alert) do
    id = "AL-" <> Integer.to_string(System.unique_integer([:positive]))
    rec = Map.put(alert, :id, id)
    Ets.insert(:hermes_alerts, {id, rec})
    {:ok, id, rec}
  end

  def alerts, do: Ets.tab2list(:hermes_alerts) |> Enum.map(fn {_k, v} -> v end)

  def ack_alert(id) do
    case Ets.lookup(:hermes_alerts, id) do
      [{^id, rec}] ->
        rec2 = Map.put(rec, :ack, true)
        Ets.insert(:hermes_alerts, {id, rec2})
        {:ok, rec2}
      [] -> {:error, :not_found}
    end
  end

  # --- missions (HERMES memory) --------------------------------------------
  def new_mission(kind, payload) do
    id = "MS-" <> Integer.to_string(System.unique_integer([:positive]))
    rec = %{id: id, kind: kind, payload: payload, frames: [], status: "running",
            started_ts: System.system_time(:second)}
    Ets.insert(:hermes_missions, {id, rec})
    {:ok, id, rec}
  end

  def add_frame(mission_id, frame) do
    case Ets.lookup(:hermes_missions, mission_id) do
      [{^id, rec}] ->
        rec2 = %{rec | frames: rec.frames ++ [frame]}
        Ets.insert(:hermes_missions, {mission_id, rec2})
        {:ok, rec2}
      [] -> {:error, :not_found}
    end
  end

  def complete_mission(mission_id, summary_sw) do
    case Ets.lookup(:hermes_missions, mission_id) do
      [{^id, rec}] ->
        rec2 = %{rec | status: "done", summary_sw: summary_sw, ended_ts: System.system_time(:second)}
        Ets.insert(:hermes_missions, {mission_id, rec2})
        {:ok, rec2}
      [] -> {:error, :not_found}
    end
  end

  def missions, do: Ets.tab2list(:hermes_missions) |> Enum.map(fn {_k, v} -> v end)

  # --- HITL -----------------------------------------------------------------
  def add_hitl(%{} = req) do
    id = "HL-" <> Integer.to_string(System.unique_integer([:positive]))
    rec = Map.merge(%{id: id, status: "pending", created_ts: System.system_time(:second)}, req)
    Ets.insert(:hermes_hitl, {id, rec})
    {:ok, id, rec}
  end

  def hitl_pending, do: Ets.tab2list(:hermes_hitl) |> Enum.map(fn {_k, v} -> v end) |> Enum.filter(&(&1.status == "pending"))

  def hitl_decide(id, decision, who) do
    case Ets.lookup(:hermes_hitl, id) do
      [{^id, rec}] ->
        rec2 = %{rec | status: decision, decided_by: who, decided_ts: System.system_time(:second)}
        Ets.insert(:hermes_hitl, {id, rec2})
        {:ok, rec2}
      [] -> {:error, :not_found}
    end
  end

  def hitl_all, do: Ets.tab2list(:hermes_hitl) |> Enum.map(fn {_k, v} -> v end)

  # --- snapshot -------------------------------------------------------------
  def snapshot do
    %{
      telemetry: all_telemetry(),
      alerts: alerts(),
      missions: missions(),
      hitl: hitl_all()
    }
  end

  defp lookup(t, k) do
    case Ets.lookup(t, k) do
      [{^k, v}] -> v
      [] -> nil
    end
  end
end
