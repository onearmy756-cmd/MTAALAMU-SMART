defmodule HermesGateway.Router do
  @moduledoc """
  HTTP API ya HERMES Gateway (port 8088).

  | Method | Path | Maelezo |
  |---|---|---|
  | GET  | /health | hali ya gateway |
  | POST | /ingest | telemetry event (device_id, fields, ts?) |
  | GET  | /telemetry | telemetry yote ya hivi punde |
  | GET  | /alerts | alerts zote |
  | POST | /alerts/:id/ack | thibitisha alert |
  | POST | /voice | matini ya Whisper → FST → amri (HITL kama inahitajika) |
  | GET  | /hitl/pending | HITL requests zinazosubiri |
  | POST | /hitl/:id/decide | approve/reject (RUHUSU / GHAIRI) |
  | GET  | /missions | missions za HERMES + frames |
  | GET  | /missions/:id | mission moja |
  | POST | /worker/register | worker (Python) anajitambulisha |
  | GET  | /worker/tasks | worker anapoll kazi |
  | POST | /worker/done | worker anaripotoa kazi |
  | POST | /engine/call | proxy ndogo kwenda engine-rust (8080) |
  """
  use Plug.Router
  require Logger

  plug(Plug.Logger)
  plug(:match)
  plug(Plug.Parsers, parsers: [:json], json_decoder: Jason, pass: ["application/json"])
  plug(:dispatch)

  get "/health" do
    send_json(conn, 200, %{
      ok: true, service: "hermes_gateway", version: "1.0.0",
      telemetry: map_size(HermesGateway.Store.all_telemetry()),
      alerts: length(HermesGateway.Store.alerts()),
      missions: length(HermesGateway.Store.missions()),
      hitl_pending: length(HermesGateway.Store.hitl_pending())
    })
  end

  post "/ingest" do
    with %{"device_id" => device_id, "fields" => fields} when is_map(fields) <- conn.body_params do
      res = HermesGateway.Ingest.ingest(device_id, fields, conn.body_params["source"] || "http", %{})
      send_json(conn, 200, Map.put(res, :ok, true))
    else
      _ -> send_json(conn, 400, %{ok: false, error: "device_id na fields zinahitajika"})
    end
  end

  get "/telemetry" do
    send_json(conn, 200, %{ok: true, telemetry: HermesGateway.Store.all_telemetry()})
  end

  get "/alerts" do
    send_json(conn, 200, %{ok: true, alerts: HermesGateway.Store.alerts()})
  end

  post "/alerts/:id/ack" do
    case HermesGateway.Store.ack_alert(id) do
      {:ok, rec} -> send_json(conn, 200, %{ok: true, alert: rec})
      {:error, :not_found} -> send_json(conn, 404, %{ok: false, error: "haipo"})
    end
  end

  post "/voice" do
    text = conn.body_params["text"] || ""
    lang = conn.body_params["lang"] || "sw"

    if text == "" do
      send_json(conn, 400, %{ok: false, error: "text inahitajika"})
    else
      res = HermesGateway.Fst.run(text)
      res2 =
        case res["command"] do
          %{} = cmd -> Map.put(res, "command_check", HermesGateway.Hitl.check_command(cmd))
          nil -> res
        end

      send_json(conn, 200, %{ok: true, lang: lang, fst: res2})
    end
  end

  get "/hitl/pending" do
    send_json(conn, 200, %{ok: true, pending: HermesGateway.Store.hitl_pending()})
  end

  post "/hitl/:id/decide" do
    decision = conn.body_params["decision"] || ""
    who = conn.body_params["who"] || "human"

    if decision in ["approve", "reject"] do
      case HermesGateway.Store.hitl_decide(id, decision, who) do
        {:ok, rec} -> send_json(conn, 200, %{ok: true, request: rec})
        {:error, :not_found} -> send_json(conn, 404, %{ok: false, error: "haipo"})
      end
    else
      send_json(conn, 400, %{ok: false, error: "decision lazima iwe approve au reject"})
    end
  end

  get "/missions" do
    send_json(conn, 200, %{ok: true, missions: HermesGateway.Store.missions()})
  end

  get "/missions/:id" do
    found = Enum.find(HermesGateway.Store.missions(), &(&1.id == id))
    if found, do: send_json(conn, 200, %{ok: true, mission: found}),
      else: send_json(conn, 404, %{ok: false, error: "haipo"})
  end

  post "/worker/register" do
    id = conn.body_params["id"] || "worker-?"
    caps = conn.body_params["capabilities"] || []
    HermesGateway.Workers.register(id, caps)
    send_json(conn, 200, %{ok: true, registered: id, capabilities: caps})
  end

  get "/worker/tasks" do
    worker = conn.params["worker"] || "unknown"
    send_json(conn, 200, %{ok: true, tasks: HermesGateway.Workers.pop_tasks(worker)})
  end

  post "/worker/done" do
    task_id = conn.body_params["task_id"]
    result = conn.body_params["result"] || %{}
    HermesGateway.Workers.complete(task_id, result)
    send_json(conn, 200, %{ok: true})
  end

  post "/engine/call" do
    path = conn.body_params["path"] || ""
    method = (conn.body_params["method"] || "GET") |> String.upcase()
    body = conn.body_params["body"] || %{}
    HermesGateway.Engine.call(method, path, body)
    |> case do
      {:ok, status, json} -> send_json(conn, status, json)
      {:error, reason} -> send_json(conn, 502, %{ok: false, error: inspect(reason)})
    end
  end

  match _ do
    send_json(conn, 404, %{ok: false, error: "siyo route"})
  end

  defp send_json(conn, status, body) do
    conn
    |> put_resp_content_type("application/json")
    |> send_resp(status, Jason.encode!(body))
  end
end
