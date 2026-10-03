defmodule HermesGateway.Application do
  @moduledoc """
  MTAALAMU SMART — HERMES Gateway.

  Supervision tree:
    - Store (ETS)        : telemetry ya hivi punde, alerts, missions, HITL requests
    - Rules (GenServer)  : rules kutoka data/iot/telemetry_rules.json
    - Mqtt (hiari)       : emqtt kama imepachikwa (toa alama kwenye mix.exs)
    - Router (Plug/Cowboy): HTTP API kwa UI (Shiny), worker (Python) na engine (Rust)
  """
  use Application

  @impl true
  def start(_type, _args) do
    port = String.to_integer(System.get_env("HERMES_GATEWAY_PORT") || "8088")
    data_dir = System.get_env("MTAALAMU_DATA") || "../../data"

    children = [
      {HermesGateway.Store, nil},
      {HermesGateway.Rules, [Path.join(data_dir, "iot/telemetry_rules.json")]},
      {HermesGateway.Fst, [Path.join(data_dir, "iot/voice_fst.json")]},
      {HermesGateway.Mqtt, nil},
      {Plug.Cowboy, scheme: :http, plug: HermesGateway.Router, options: [port: port, ip: {0, 0, 0, 0}]}
    ]

    opts = [strategy: :one_for_one, name: HermesGateway.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
