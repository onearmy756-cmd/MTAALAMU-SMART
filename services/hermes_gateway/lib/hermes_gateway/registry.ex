defmodule HermesGateway.Registry do
  @moduledoc """
  Inasoma `data/iot/` (verticals, registry za clone, devices, agents_oss, hermes, voice_fst)
  na kutoa **summary moja** kwa UI (Shiny — tab IoT + HERMES) na clients nyingine.

  Kanuni: JSON ndio chanzo pekee (KANUNI 2/5) — module hii haitoi data kwa mkono.
  """
  use GenServer

  def start_link(opts), do: GenServer.start_link(__MODULE__, opts, name: __MODULE__)

  @impl true
  def init([data_dir]) do
    {:ok, %{data_dir: data_dir}}
  end

  @doc "Summary kamili ya IoT + HERMES kutoka JSON."
  def summary, do: GenServer.call(__MODULE__, :summary)

  @impl true
  def handle_call(:summary, _from, %{data_dir: data_dir} = state) do
    {:reply, build(data_dir), state}
  end

  defp build(data_dir) do
    verticals = read_json(Path.join(data_dir, "iot/verticals.json")) |> Map.get("verticals", [])

    registry =
      data_dir
      |> Path.join("iot/registry/*.json")
      |> Path.wildcard()
      |> Enum.sort()
      |> Enum.map(fn path ->
        doc = read_json(path)
        %{
          "vertical" => Path.basename(path, ".json"),
          "projects" => Map.get(doc, "projects", [])
        }
      end)

    devices = read_json(Path.join(data_dir, "iot/devices.json")) |> Map.get("devices", [])
    agents = read_json(Path.join(data_dir, "iot/agents_oss.json")) |> Map.get("agents", [])
    fst = read_json(Path.join(data_dir, "iot/voice_fst.json"))
    hermes = read_json(Path.join(data_dir, "iot/hermes.json")) |> Map.get("hermes", %{})

    %{
      ok: true,
      verticals: verticals,
      registry: registry,
      projects_total: registry |> Enum.map(&length(&1["projects"])) |> Enum.sum(),
      devices: devices,
      devices_total: length(devices),
      agents: agents,
      agents_total: length(agents),
      voice: %{
        "wake_words" => Map.get(fst, "wake_words", []),
        "intents_total" => fst |> Map.get("intents", []) |> length(),
        "targets_total" => fst |> Map.get("targets", []) |> length()
      },
      hermes: %{
        "pipelines" => hermes |> Map.get("pipelines", []) |> Enum.map(& &1["id"]),
        "hitl_gates" => hermes |> Map.get("hitl_gates", []) |> Enum.map(& &1["id"]),
        "upstream_repo" => get_in(hermes, ["upstream", "repo"])
      }
    }
  end

  defp read_json(path) do
    case File.read(path) do
      {:ok, body} ->
        case Jason.decode(body) do
          {:ok, doc} -> doc
          _ -> %{}
        end
      _ ->
        %{}
    end
  end
end
