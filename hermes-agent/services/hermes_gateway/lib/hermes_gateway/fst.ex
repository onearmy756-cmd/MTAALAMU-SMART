defmodule HermesGateway.Fst do
  @moduledoc """
  Sauti: Whisper (STT, upande wa Python worker) → matini → FST (finite-state transducer)
  kutoka data/iot/voice_fst.json. Grammar ya amri kwa Kiswahili bila LLM (offline).

  Token maalum: `<value>` (namba/kipimo) na `*` (nyingine yote). Regex `a|b` inaruhusiwa.
  """
  use GenServer

  def start_link(opts), do: GenServer.start_link(__MODULE__, opts, name: __MODULE__)

  @impl true
  def init([path]) do
    doc =
      case File.read(path) do
        {:ok, body} ->
          case Jason.decode(body) do
            {:ok, d} -> d
            _ -> %{}
          end
        _ -> %{}
      end
    {:ok, %{doc: doc}}
  end

  @doc "Matini (kutoka Whisper) → matokeo ya FST: state ya mwisho, amri, majibu."
  def run(text), do: GenServer.call(__MODULE__, {:run, text})

  @impl true
  def handle_call({:run, text}, _from, state) do
    doc = state.doc
    wake = doc["wake_words"] || []
    tokens = tokenize(text)
    {:reply, run_fst(doc, wake, tokens), state}
  end

  defp tokenize(text) do
    text
    |> String.downcase()
    |> String.replace(",", " , ")
    |> String.split(~r/\s+/, trim: true)
  end

  defp run_fst(_doc, _wake, []), do: empty_result("hakuna matini")

  defp run_fst(doc, wake, tokens) do
    states = get_in(doc, ["fst", "states"]) || []
    initial = get_in(doc, ["fst", "initial"]) || "START"
    accept = get_in(doc, ["fst", "accept"]) || []
    transitions = get_in(doc, ["fst", "transitions"]) || []

    {final, replies, path, value} =
      Enum.reduce(tokens, {initial, [], [], nil}, fn tok, {state, reps, path, val} ->
        trans =
          Enum.find(transitions, fn t ->
            t["from"] == state and token_matches?(t["token"], tok)
          end)

        case trans do
          nil ->
            {state, reps, path ++ [{state, tok, nil}], val}

          t ->
            val =
              if t["token"] == "<value>",
                do: parse_value(tok),
                else: val

            reply = t["reply_sw"]
            reps = if reply, do: reps ++ [reply], else: reps
            {t["to"], reps, path ++ [{state, tok, t["to"]}], val}
        end
      end)

    intents = doc["intents"] || []
    targets = doc["targets"] || []

    spoken = Enum.filter(tokens, &(&1 not in wake))
    intent = Enum.find_value(spoken, fn t -> Enum.find(intents, &(&1["sw"] == t)) end)
    target = Enum.find_value(spoken, fn t -> Enum.find(targets, &(&1["sw"] == t)) end)

    accepted = final in accept

    command =
      if accepted and intent and target do
        %{
          "intent" => intent["action"],
          "device" => target["device"],
          "actuator" => target["actuator"],
          "value" => value
        }
      else
        nil
      end

    %{
      "ok" => accepted,
      "final_state" => final,
      "valid_states" => states,
      "accepted" => accepted,
      "path" => path,
      "replies_sw" => replies,
      "intent" => if(intent, do: intent["action"]),
      "target" => if(target, do: target["device"]),
      "command" => command,
      "error" => if(accepted, do: nil, else: "amri haikufika mwisho wa grammar")
    }
  end

  defp token_matches?("<value>", tok), do: Regex.match?(~r/^-?\d+(\.\d+)?$/, tok)
  defp token_matches?("*", _tok), do: true
  defp token_matches?(pattern, tok) when is_binary(pattern) do
    alts = String.split(pattern, "|")
    tok in alts or Regex.match?(~r/^#{Regex.escape(pattern)}$/, tok)
  end
  defp token_matches?(_, _), do: false

  defp parse_value(tok) do
    case Float.parse(tok) do
      {v, ""} -> v
      {v, _} -> v
      :error -> nil
    end
  end

  defp empty_result(reason) do
    %{"ok" => false, "accepted" => false, "final_state" => "START", "path" => [],
      "replies_sw" => [], "intent" => nil, "target" => nil, "command" => nil,
      "error" => reason}
  end
end
