defmodule HermesGatewayTest do
  use ExUnit.Case, async: false
  alias HermesGateway.{Rules, Fst, Ingest, Store, Registry}

  setup_all do
    # Rules/Fst zinaanzishwa na Application kupitia test_helper (bila cowboy port conflict:
    # Application.ex inapakia data kutoka MTAALAMU_DATA ya env au default ../../data)
    :ok
  end

  describe "rules engine" do
    test "hr ndani ya kikomo = GOOD" do
      res = Rules.evaluate("afya.patient_monitor", %{"hr_bpm" => 75})
      [%{"status" => status} | _] = res["results"]
      assert status == "GOOD"
    end

    test "hr ya juu = WARNING" do
      res = Rules.evaluate("afya.patient_monitor", %{"hr_bpm" => 130})
      [%{"status" => status} | _] = res["results"]
      assert status == "WARNING"
    end

    test "spo2 chini sana = FAIL + alert" do
      res = Rules.evaluate("afya.patient_monitor", %{"spo2_pct" => 80})
      statuses = Enum.map(res["results"], & &1["status"])
      assert "FAIL" in statuses
      assert length(res["alerts"]) > 0
    end

    test "udongo mkavu = FAIL" do
      res = Rules.evaluate("kilimo.soil_sensor", %{"soil_moisture_pct" => 10})
      statuses = Enum.map(res["results"], & &1["status"])
      assert "FAIL" in statuses
    end

    test "device isiyo na rules = matokeo matupu" do
      res = Rules.evaluate("afya.hakuna", %{"x" => 1})
      assert res["results"] == []
    end
  end

  describe "fst (sauti)" do
    test "amri kamili: hermes washa taa" do
      res = Fst.run("hermes washa taa")
      assert res["accepted"] == true
      assert res["intent"] == "turn_on"
      assert res["target"] == "nyumbani.smart_light"
      assert res["command"]["device"] == "nyumbani.smart_light"
    end

    test "bila wake word = rejected" do
      res = Fst.run("washa taa")
      assert res["accepted"] == false
    end

    test "amri ya valve inafika mwisho" do
      res = Fst.run("hermes fungua valve sasa")
      assert res["accepted"] == true
      assert res["intent"] == "open"
      assert res["target"] == "kilimo.irrigation_valve"
    end
  end

  describe "registry (iot summary)" do
    setup do
      {:ok, _} = Registry.start_link([Path.expand("../../data", __DIR__)])
      :ok
    end

    test "verticals 4 + projects + devices + agents kutoka JSON" do
      s = Registry.summary()
      assert length(s.verticals) == 4
      assert s.projects_total >= 30
      assert s.devices_total >= 12
      assert s.agents_total >= 20
      ids = Enum.map(s.verticals, & &1["id"])
      assert "afya" in ids
      assert "usalama" in ids
      assert "nyumbani" in ids
      assert "kilimo" in ids
    end

    test "hermes + voice summary zipo" do
      s = Registry.summary()
      assert "iot_incident" in s.hermes["pipelines"]
      assert length(s.hermes["hitl_gates"]) >= 4
      assert "hermes" in s.voice["wake_words"]
    end
  end

  describe "ingest + missions" do
    test "telemetry FAIL inaanzisha mission (HERMES memory)" do
      res = Ingest.ingest("afya.patient_monitor", %{"spo2_pct" => 80}, "test")
      assert res["mission_id"] != nil

      missions = Store.missions()
      assert Enum.any?(missions, &(&1.id == res["mission_id"]))
    end

    test "telemetry GOOD haina mission" do
      res = Ingest.ingest("nyumbani.temp_humidity", %{"temp_c" => 24, "humidity_pct" => 50}, "test")
      assert res["mission_id"] == nil
    end
  end
end
