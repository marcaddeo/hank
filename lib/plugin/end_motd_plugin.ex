defmodule Hank.Plugin.EndMotdPlugin do
  use Hank.Core.Plugin
  alias Hank.Core.Client.State
  alias Hank.Core.Client.Server, as: Client

  def handle_cast({_, %State{channels: channels, password: password}}, state) do
    if password do
      Client.identify(password)
    end

    Enum.each(channels, fn (channel) ->
      Client.join(channel)
    end)
    {:noreply, state}
  end
end
