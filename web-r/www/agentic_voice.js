/* Agentic Vision — TTS Kiswahili (Web Speech API) */
(function () {
  function pickSwVoice() {
    if (!window.speechSynthesis) return null;
    var voices = speechSynthesis.getVoices() || [];
    var sw = voices.filter(function (v) {
      return (v.lang || "").toLowerCase().indexOf("sw") === 0;
    });
    return sw[0] || voices[0] || null;
  }

  window.mtaalamuSpeak = function (text, opts) {
    opts = opts || {};
    if (!window.speechSynthesis || !text) return;
    try {
      speechSynthesis.cancel();
      var u = new SpeechSynthesisUtterance(String(text));
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
    var t = el ? el.innerText || el.textContent : "";
    if (!t) t = document.getElementById("av_voice_text")
      ? document.getElementById("av_voice_text").innerText
      : "Habari. Mimi ni Mtaalamu Smart.";
    // Speak first ~500 chars to avoid very long utterances
    window.mtaalamuSpeak(t.slice(0, 500));
  };

  // Chrome loads voices async
  if (window.speechSynthesis) {
    speechSynthesis.onvoiceschanged = function () {};
  }
})();
