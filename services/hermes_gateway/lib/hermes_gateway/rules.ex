defmodule HermesGateway.Rules do
  @moduledoc """
  Inasoma data/iot/telemetry_rules.json na kutekeleza checks kwenye telemetry events.
  Grammar: `v` = thamani; `+ - * / %`, comparisons, `&& || !`, `true/false`.
  Sheria ya kwanza inayolingana inashinda (kama data/SCHEMA.md inavyotaka).
  """
  use GenServer

  def start_link(opts), do: GenServer.start_link(__MODULE__, opts, name: __MODULE__)

  @impl true
  def init([path]) do
    rules =
      case File.read(path) do
        {:ok, body} ->
          case Jason.decode(body) do
            {:ok, %{"rules" => rs}} -> rs
            _ -> []
          end
        _ -> []
      end
    {:ok, %{rules: rules}}
  end

  def evaluate(device_id, fields), do: GenServer.call(__MODULE__, {:evaluate, device_id, fields})

  @impl true
  def handle_call({:evaluate, device_id, fields}, _from, state) do
    results =
      for rule <- state.rules, rule["device"] == device_id, field = rule["field"],
          Map.has_key?(fields, field) do
        v = fields[field]
        matched = Enum.find(rule["checks"] || [], fn c -> check?(c["check"] || "true", v) end)
        if matched do
          %{"rule" => rule["id"], "field" => field, "value" => v,
            "status" => matched["status"], "msg" => matched["msg"],
            "alert" => rule["alert"]}
        else
          %{"rule" => rule["id"], "field" => field, "value" => v, "status" => "INFO"}
        end
      end

    # Alerti kwa FAIL tu (WARNING inaonekana kwenye telemetry panel; FAIL inaingia kwenye alert flow)
    alerts =
      results
      |> Enum.filter(&(&1["status"] == "FAIL"))
      |> Enum.map(fn r ->
        {:ok, id, rec} = HermesGateway.Store.add_alert(%{
          device: device_id, field: r["field"], value: r["value"],
          msg: r["msg"], alert: r["alert"] || %{}
        })
        Map.put(rec, :id, id)
      end)

    {:reply, %{"results" => results, "alerts" => alerts}, state}
  end

  # --- checker ndogo (shared grammar, salama kwa untrusted JSON) -------------
  # Hakuna Code.eval: tokenizer + parser + evaluator ndogo pekee.
  defp check?("true", _v), do: true
  defp check?("false", _v), do: false
  defp check?(expr, v) do
    expr
    |> String.trim()
    |> tokenize()
    |> parse_or()
    |> eval_or(%{"v" => v})
  rescue
    _ -> false
  end

  defp tokenize(s) do
    s
    |> String.replace("(", " ( ")
    |> String.replace(")", " ) ")
    |> String.replace("&&", " && ")
    |> String.replace("||", " || ")
    |> String.replace("!", " ! ")
    |> String.replace("<=", " <= ")
    |> String.replace(">=", " >= ")
    |> String.replace("==", " == ")
    |> String.replace("!=", " != ")
    |> String.replace("<", " < ")
    |> String.replace(">", " > ")
    |> String.split(~r/\s+/, trim: true)
  end

  # Parser rahisi: or → and → cmp → add → unary → primary
  defp parse_or(tokens) do
    {left, rest} = parse_and(tokens)
    case rest do
      ["||" | t] -> {l2, r2} = parse_or(t); {:or, left, l2, r2}
      _ -> {:or, left, nil, rest}
    end
  end
  defp parse_and(tokens) do
    {left, rest} = parse_cmp(tokens)
    case rest do
      ["&&" | t] -> {l2, r2} = parse_and(t); {:and, left, l2, r2}
      _ -> {:and, left, nil, rest}
    end
  end
  defp parse_cmp(tokens) do
    {left, rest} = parse_add(tokens)
    case rest do
      [op | t] when op in ["<", "<=", ">", ">=", "==", "!="] ->
        {right, r2} = parse_add(t)
        {:cmp, String.to_atom(op), left, right, r2}
      _ -> {:cmp, nil, left, nil, rest}
    end
  end
  defp parse_add(tokens) do
    {left, rest} = parse_unary(tokens)
    parse_add_loop(left, rest)
  end
  defp parse_add_loop(left, [op | t]) when op in ["+", "-"] do
    {right, r2} = parse_unary(t)
    parse_add_loop({:bin, op, left, right}, r2)
  end
  defp parse_add_loop(left, rest), do: {left, rest}
  defp parse_unary(["!" | t]) do
    {inner, rest} = parse_unary(t)
    {:not, inner, rest}
  end
  defp parse_unary(["-" | t]) do
    {inner, rest} = parse_unary(t)
    {:neg, inner, rest}
  end
  defp parse_unary(tokens) do
    case tokens do
      ["(" | t] ->
        {inner, rest} = parse_or(t)
        case rest do
          [")" | r2] -> {inner, r2}
          _ -> {inner, rest}
        end
      [tok | rest] ->
        cond do
          tok == "true" -> {true, rest}
          tok == "false" -> {false, rest}
          Regex.match?(~r/^-?\d+\.\d+$|^-?\d+$/, tok) -> {String.to_float(to_float_str(tok)), rest}
          true -> {:var, tok, rest}
        end
    end
  end
  defp to_float_str(s) do
    if String.contains?(s, "."), do: s, else: s <> ".0"
  end

  defp eval_or(left, nil, env), do: truthy?(eval(left, env))
  defp eval_or(left, right, env) do
    a = eval(left, env)
    if truthy?(a), do: true, else: truthy?(eval(right, env))
  end
  defp eval(left, env) do
    case left do
      nil -> false
      node -> eval_node(node, env)
    end
  end
  defp truthy?(v), do: v == true

  defp eval_node({:or, left, right, _rest}, env) do
    if truthy?(eval(left, env)), do: true, else: eval(right, env)
  end
  defp eval_node({:and, left, right, _rest}, env) do
    if truthy?(eval(left, env)), do: eval(right, env), else: false
  end
  defp eval_node({:cmp, nil, left, _right, _rest}, env), do: eval(left, env)
  defp eval_node({:cmp, op, left, right, _rest}, env) do
    a = eval(left, env); b = eval(right, env)
    case op do
      :"<" -> a < b; :"<=" -> a <= b; :">" -> a > b; :">=" -> a >= b
      :"==" -> a == b; :"!=" -> a != b
    end
  end
  defp eval_node({:bin, op, l, r}, env) do
    a = eval(l, env); b = eval(r, env)
    case op do
      "+" -> a + b; "-" -> a - b
    end
  end
  defp eval_node({:not, inner, _rest}, env), do: not eval(inner, env)
  defp eval_node({:neg, inner, _rest}, env), do: -eval(inner, env)
  defp eval_node({:var, "v", _rest}, env), do: env["v"]
  defp eval_node({:var, _name, _rest}, _env), do: raise("unknown variable")
  defp eval_node(v, _env) when is_boolean(v), do: v
  defp eval_node(v, _env) when is_number(v), do: v
  defp eval_node({:lit, v}, _env), do: v
end
