defmodule Akili.CLI do
  @moduledoc """
  CLI ndogo: `mix run -e 'Akili.CLI.main(["Habari", "yako"])'`.
  Pia hutumika kama mfano wa matumizi ya `Akili.Port`.
  """

  def main(args \\ []) do
    {:ok, _} = Application.ensure_all_started(:akili)
    prompt = Enum.join(args, " ")

    reply = Akili.Port.chat([%{role: "user", content: prompt}], max_tokens: 256)

    case reply do
      %{"ok" => true, "text" => text} ->
        IO.puts(text)

      other ->
        IO.puts(:stderr, "kosa: #{inspect(other)}")
        System.halt(1)
    end
  end
end
