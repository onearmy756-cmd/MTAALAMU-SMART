defmodule Akili.Application do
  @moduledoc false
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Akili.Port, []}
    ]

    Supervisor.start_link(children, strategy: :one_for_one, name: Akili.Supervisor)
  end
end
