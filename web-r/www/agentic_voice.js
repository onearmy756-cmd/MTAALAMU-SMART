/* Agentic Vision — TTS Kiswahili */
(function () {
  function pickSwVoice() {
    if (!window.speechSynthesis) return null;
    var voices = speechSynthesis.getVoices() || [];
    var sw = voices.filter(function (v) {
      return (v.lang || "").toLowerCase().indexOf("sw") === 0;
    });
    return sw[0] || null;
  }

  window.mtaalamuSpeak = function (text, opts) {
    opts = opts || {};
    if (!window.speechSynthesis || !text) return;
    try {
      speechSynthesis.cancel();
      var u = new SpeechSynthesisUtterance(String(text).slice(0, 600));
      u.lang = opts.lang || "sw";
      u.rate = opts.rate || 0.95;
      u.pitch = opts.pitch || 1;
      var v = pickSwVoice();
      if (v) u.voice = v;
      speechSynthesis.speak(u);
    } catch (e) {
      console.warn("TTS", e);
    }
  };

  window.mtaalamuSpeakLog = function (selector) {
    var el = document.querySelector(selector || "#av_narration_text");
    var t = el ? (el.innerText || el.textContent || "") : "";
    if (!t) {
      var v = document.getElementById("av_voice_text");
      t = v ? v.innerText : "Habari. Mimi ni Mtaalamu Smart.";
    }
    window.mtaalamuSpeak(t);
  };

  function register() {
    if (window.Shiny && Shiny.addCustomMessageHandler) {
      Shiny.addCustomMessageHandler("mtaalamu_speak", function (msg) {
        if (msg && msg.text) window.mtaalamuSpeak(msg.text, msg);
      });
    }
  }
  register();
  document.addEventListener("DOMContentLoaded", register);
  if (window.speechSynthesis) speechSynthesis.onvoiceschanged = function () {};
})();
