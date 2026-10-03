defmodule HermesGateway.Mqtt do
  @moduledoc """
  MQTT ingest (HIARI). Kama emqtt ipo (toa alama kwenye mix.exs) na broker inapatikana,
  gateway inasikiliza `mtaalamu/+/+/telemetry` na kuingiza events kwenye Rules.

  Bila emqtt: GenServer hii inatangaza tu kuwa MQTT imezimwa — HTTP /ingest inafanya kazi
  kikamilifu (KANUNI: offline-first, hakuna dependency ngumu).
  """
  use GenServer
  require Logger

  def start_link(_arg), do: GenServer.start_link(__MODULE__, nil, name: __MODULE__)

  @impl true
  def init(_arg) do
    case Code.ensure_loaded(:emqtt) do
      {:module, _} ->
        host = System.get_env("MQTT_HOST") || "127.0.0.1"
        port = String.to_integer(System.get_env("MQTT_PORT") || "1883")

        {:ok, pid} =
          :emqtt.start_link(
            host: String.to_charlist(host),
            port: port,
            clientid: "hermes_gateway",
            name: :hermes_emqtt
          )

        {:ok, _props} = :emqtt.connect(pid)
        {:ok, _rc, _props} = :emqtt.subscribe(pid, {"mtaalamu/+/+/telemetry", 1})
        Logger.info("MQTT imeunganishwa: #{host}:#{port}")
        {:ok, %{connected: true}}

      {:error, _} ->
        Logger.warning("emqtt haipo — MQTT imezimwa; tumia HTTP /ingest (offline-first)")
        {:ok, %{connected: false}}
    end
  end

  @impl true
  def handle_info({:publish, %{topic: topic, payload: payload}}, state) do
    case Jason.decode(payload) do
      {:ok, %{"device_id" => device_id} = event} ->
        fields = event["fields"] || %{}
        HermesGateway.Ingest.ingest(device_id, fields, "mqtt", %{"topic" => topic})
        {:noreply, state}

      _ ->
        Logger.warning("MQTT payload si JSON ya event: #{topic}")
        {:noreply, state}
    end
  end

  def handle_info(_msg, state), do: {:noreply, state}
end
