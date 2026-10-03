defmodule HermesGateway.Workers do
  @moduledoc """
  Workers (LangChain/Whisper/Crawl4AI — Python) zina:
    1. `POST /worker/register` — kujitambulisha na capabilities
    2. `GET  /worker/tasks?worker=<id>` — kupoll kazi
    3. `POST /worker/done` — kuripotoa matokeo

  HERMES anagawa kazi kwa workers; matokeo yanaingia kwenye mission frames (memory).
  """

  def register(id, caps) do
    :ets.insert(:hermes_workers, {id, %{capabilities: caps, registered_ts: System.system_time(:second)}})
    :ok
  end

  def push_task(worker_id, task) do
    key = {:tasks, worker_id}
    current = case :ets.lookup(:hermes_workers, key) do
      [{^key, list}] when is_list(list) -> list
      _ -> []
    end
    :ets.insert(:hermes_workers, {key, current ++ [task]})
    {:ok, task["task_id"]}
  end

  def pop_tasks(worker_id) do
    key = {:tasks, worker_id}
    case :ets.lookup(:hermes_workers, key) do
      [{^key, list}] when is_list(list) ->
        :ets.insert(:hermes_workers, {key, []})
        list
      _ -> []
    end
  end

  def complete(task_id, result) do
    # Frame ya mission kama task ina mission_id
    case result["mission_id"] do
      mid when is_binary(mid) ->
        HermesGateway.Store.add_frame(mid, %{
          agent: result["agent"] || "worker",
          step: result["step"] || "done",
          ts: System.system_time(:second),
          narration_sw: result["summary_sw"] || "Kazi ya worker imekamilika.",
          evidence: result
        })
      _ -> :ok
    end

    :ets.insert(:hermes_tasks_done, {task_id, %{result: result, ts: System.system_time(:second)}})
    :ok
  end
end
