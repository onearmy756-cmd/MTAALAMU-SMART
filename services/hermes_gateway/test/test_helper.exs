ExUnit.start()

# Anza subsystem za gateway kwa tests (bila Cowboy/HTTP na bila MQTT):
{:ok, _} = HermesGateway.Store.start_link(nil)
{:ok, _} = HermesGateway.Rules.start_link([Path.expand("../../data/iot/telemetry_rules.json", __DIR__)])
{:ok, _} = HermesGateway.Fst.start_link([Path.expand("../../data/iot/voice_fst.json", __DIR__)])

# ETS za workers (kama Application haitaanza Cowboy kwenye test env)
:ets.new(:hermes_workers, [:named_table, :set, :public])
:ets.new(:hermes_tasks_done, [:named_table, :set, :public])
