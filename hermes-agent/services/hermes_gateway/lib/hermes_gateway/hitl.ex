defmodule HermesGateway.Hitl do
  @moduledoc """
  HITL (Human-In-The-Loop) — KANUNI 4 ya mradi.

  Amri ya actuator/device inayogusa: afya (actuation), locks, sirens, valves, pampu au
  mzigo mkubwa wa umeme — lazima ipite gate kabla ya kutumwa. Gate zinachukuliwa kutoka
  `data/iot/hermes.json` (hitl_gates) na `data/iot/devices.json` (hitl_required).
  """

  @data_dir System.get_env("MTAALAMU_DATA") || "../../data"

  def check_command(%{} = cmd) do
    device = cmd["device"] || ""
    vertical = device |> String.split(".") |> List.first()

    gates = hermes_gates()
    gate =
      Enum.find(gates, fn g ->
        cond do
          vertical == "afya" and String.contains?(g["applies"], "afya") -> true
          vertical == "usalama" and (String.contains?(g["applies"], "lock") or String.contains?(g["applies"], "siren")) -> true
          vertical == "kilimo" and (String.contains?(g["applies"], "valve") or String.contains?(g["applies"], "pump")) -> true
          vertical == "nyumbani" and String.contains?(g["applies"], "umeme") -> cmd["value"] != nil
          true -> false
        end
      end)

    device_hitl = device_hitl_required?(device)

    if gate or device_hitl do
      {:ok, id, rec} =
        HermesGateway.Store.add_hitl(%{
          kind: "actuation",
          device: device,
          actuator: cmd["actuator"],
          value: cmd["value"],
          gate: if(gate, do: gate["id"], else: "device-level"),
          reason: if(gate, do: gate["reason_sw"], else: "Kifaa kinahitaji idhini (hitl_required)"),
          source: "voice_fst"
        })

      %{required: true, hitl_id: id, gate: rec.gate, reason: rec.reason, status: "awaiting_approval"}
    else
      %{required: false, status: "auto_allowed"}
    end
  end

  defp hermes_gates do
    path = Path.join(@data_dir, "iot/hermes.json")
    case File.read(path) do
      {:ok, body} ->
        case Jason.decode(body) do
          {:ok, %{"hermes" => %{"hitl_gates" => gates}}} -> gates
          _ -> []
        end
      _ -> []
    end
  end

  defp device_hitl_required?(device_id) do
    path = Path.join(@data_dir, "iot/devices.json")
    case File.read(path) do
      {:ok, body} ->
        case Jason.decode(body) do
          {:ok, %{"devices" => devices}} ->
            Enum.any?(devices, &(&1["id"] == device_id and &1["hitl_required"] == true))
          _ -> false
        end
      _ -> false
    end
  end
end
