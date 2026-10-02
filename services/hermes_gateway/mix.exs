defmodule HermesGateway.MixProject do
  use Mix.Project

  def project do
    [
      app: :hermes_gateway,
      version: "1.0.0",
      elixir: "~> 1.15",
      elixirc_paths: elixirc_paths(Mix.env()),
      start_permanent: Mix.env() == :prod,
      description: "MTAALAMU SMART — HERMES Gateway: MQTT ingest, rules, FST voice, missions, HITL",
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger, :inets, :ssl, :crypto],
      mod: {HermesGateway.Application, []}
    ]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]

  # MQTT (emqtt) ni HIARI: toa alama (#) kwenye mstari wa emqtt ukihitaji broker halisi.
  # Bila emqtt, gateway inaendelea na HTTP ingest + demo simulator (KANUNI: offline-first).
  defp deps do
    [
      {:jason, "~> 1.4"},
      {:plug_cowboy, "~> 2.7"}
      # {:emqtt, github: "emqx/emqtt", tag: "1.13.3"}
    ]
  end
end
