defmodule Hank.Core.Client.State do
  @moduledoc """
  The state of the client
  """

  defstruct [
    channels:    [],
    nickname:    nil,
    password:    nil,
    realname:    nil,
    plugins:     [],
  ]
end
