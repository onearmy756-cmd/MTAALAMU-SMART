defmodule Akili.Port do
  @moduledoc """
  Kiungo kati ya Elixir na daemon ya Rust (`akilid serve`).

  Daemon huzungumza JSON-lines kwenye stdin/stdout. Moduli hii huifungua mara moja,
  huiweka kama process ya OTP, na hutoa API rahisi kwa sehemu nyingine za MTECH OS.
  """

  use GenServer
  require Logger

  @timeout 600_000

  # ---------- API ----------

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @doc "Ombi la jumla la JSON; hurudisha map ya jibu."
  def call(request, timeout \\ @timeout) do
    GenServer.call(__MODULE__, {:request, request}, timeout)
  end

  def health, do: call(%{op: "health"}, 60_000)

  @doc "Mazungumzo. `messages` ni orodha ya `%{role: ..., content: ...}`."
  def chat(messages, opts \\ []) do
    req =
      %{
        op: "chat",
        messages: messages,
        system: opts[:system],
        images: opts[:images] || [],
        max_tokens: opts[:max_tokens] || 512,
        temperature: opts[:temperature] || 0.2,
        top_p: opts[:top_p] || 0.9,
        remember: opts[:remember] || false
      }
      |> drop_nils()

    call(req)
  end

  @doc "Hifadhi kumbukumbu."
  def remember(text, opts \\ []) do
    call(
      drop_nils(%{
        op: "remember",
        text: text,
        kind: opts[:kind] || "ukweli",
        tags: opts[:tags] || [],
        salience: opts[:salience] || 1.0
      })
    )
  end

  @doc "Tafuta kumbukumbu."
  def recall(query, limit \\ 8), do: call(%{op: "recall", query: query, limit: limit})

  @doc "Muhtasari wa kumbukumbu zote."
  def digest(limit \\ 2000), do: call(%{op: "digest", limit: limit})

  defp drop_nils(map), do: Map.reject(map, fn {_k, v} -> is_nil(v) end)

  # ---------- GenServer ----------

  @impl true
  def init(opts) do
    binary = opts[:binary] || System.get_env("AKILID_BIN") || "akilid"
    model = opts[:model] || System.get_env("AKILI_MODEL")
    dtype = opts[:dtype] || System.get_env("AKILI_DTYPE") || "f16"
    memory = opts[:memory] || System.get_env("AKILI_MEMORY") || "data/kumbukumbu.jsonl"

    if is_nil(model) do
      raise "AKILI_MODEL haipo; weka njia ya modeli ya Qwen2.5-VL"
    end

    args = [
      "serve",
      "--model",
      model,
      "--dtype",
      dtype,
      "--memory",
      memory
    ]

    port =
      Port.open({:spawn_executable, System.find_executable(binary) || binary}, [
        :binary,
        :exit_status,
        :use_stdio,
        :stream,
        {:args, args},
        {:line, 8_000_000}
      ])

    {:ok, %{port: port, pending: :queue.new(), binary: binary, model: model}}
  end

  @impl true
  def handle_call({:request, req}, from, state) do
    line = Jason.encode!(req) <> "\n"
    true = Port.command(state.port, line)
    {:noreply, %{state | pending: :queue.in(from, state.pending)}}
  end

  @impl true
  def handle_info({port, {:data, {:eol, line}}}, %{port: port} = state) do
    case :queue.out(state.pending) do
      {{:value, from}, rest} ->
        reply =
          case Jason.decode(line) do
            {:ok, map} -> map
            {:error, _} -> %{"ok" => false, "error" => "jibu la Rust si JSON sahihi"}
          end

        GenServer.reply(from, reply)
        {:noreply, %{state | pending: rest}}

      {:empty, _} ->
        Logger.warning("akilid: jibu bila ombi: #{line}")
        {:noreply, state}
    end
  end

  def handle_info({port, {:data, {:noeol, chunk}}}, %{port: port} = state) do
    Logger.debug("akilid stderr/chunk: #{chunk}")
    {:noreply, state}
  end

  def handle_info({port, {:exit_status, status}}, %{port: port} = state) do
    Logger.error("akilid imeondoka (status #{status}); inaanzishwa upya")
    {:stop, {:akilid_exit, status}, state}
  end

  def handle_info(_msg, state), do: {:noreply, state}

  @impl true
  def terminate(_reason, %{port: port}) do
    if Port.info(port), do: Port.close(port)
    :ok
  end
end
