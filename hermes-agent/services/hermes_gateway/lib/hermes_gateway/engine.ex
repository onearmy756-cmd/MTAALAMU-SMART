defmodule HermesGateway.Engine do
  @moduledoc """
  Proxy ndogo kwenda engine-rust (HTTP, port 8080 chaguo-msingi).

  KANUNI R-1: LLM haihesabu — HERMES na workers wanaita engine ya Rust kwa hesabu zote
  (formulas, bayes, knowledge). Endpoints zilizopo kwenye engine: /api/diagnose,
  /api/formula/calculate (ona PLAN.md).
  """

  def call(method, path, body \\ %{}) do
    base = System.get_env("MTAALAMU_ENGINE_URL") || "http://127.0.0.1:8080"
    url = base <> path

    headers = [{"content-type", "application/json"}]
    request =
      case method do
        "GET" -> {String.to_charlist(url), headers}
        _ -> {String.to_charlist(url), headers, ~c"application/json", Jason.encode!(body)}
      end

    case :httpc.request(method, request, [timeout: 10_000], []) do
      {:ok, {{_, status, _}, _hdrs, resp_body}} ->
        case Jason.decode(to_string(resp_body)) do
          {:ok, json} -> {:ok, status, json}
          _ -> {:ok, status, %{"raw" => to_string(resp_body)}}
        end

      {:error, reason} ->
        {:error, reason}
    end
  end
end
