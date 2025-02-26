import { useState, useEffect, useRef } from 'react';

interface TTSOptions {
  rate?: number;
  pitch?: number;
  volume?: number;
  voice?: SpeechSynthesisVoice;
}

export const useTTS = () => {
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [utterance, setUtterance] = useState<SpeechSynthesisUtterance | null>(null);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const completionCheckRef = useRef<NodeJS.Timeout | null>(null);

  // Clean up function to clear all timeouts and reset state
  const cleanup = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    if (completionCheckRef.current) {
      clearTimeout(completionCheckRef.current);
      completionCheckRef.current = null;
    }
  };

  // Handle component unmounting
  useEffect(() => {
    return () => {
      if (utterance) {
        speechSynthesis.cancel();
      }
      cleanup();
    };
  }, [utterance]);

  // Speak function with improved error handling
  const speak = (text: string, options?: TTSOptions) => {
    // Cancel any ongoing speech
    stop();
    
    try {
      const newUtterance = new SpeechSynthesisUtterance(text);
      
      // Apply options
      if (options) {
        if (options.rate !== undefined) newUtterance.rate = options.rate;
        if (options.pitch !== undefined) newUtterance.pitch = options.pitch;
        if (options.volume !== undefined) newUtterance.volume = options.volume;
        if (options.voice !== undefined) newUtterance.voice = options.voice;
      }
      
      // Handle events
      newUtterance.onstart = () => {
        setIsSpeaking(true);
        setIsPaused(false);
        
        // Chrome bug workaround - sometimes speechSynthesis stops unexpectedly
        // Set interval to restart if it cuts off
        const resumeIfNeeded = () => {
          if (isSpeaking && !isPaused && !speechSynthesis.speaking) {
            speechSynthesis.resume();
          }
        };
        
        timeoutRef.current = setInterval(resumeIfNeeded, 250) as unknown as NodeJS.Timeout;
      };
      
      newUtterance.onpause = () => setIsPaused(true);
      newUtterance.onresume = () => setIsPaused(false);
      
      newUtterance.onend = () => {
        setIsSpeaking(false);
        setIsPaused(false);
        cleanup();
      };
      
      newUtterance.onerror = (event) => {
        console.error('TTS Error:', event);
        setIsSpeaking(false);
        setIsPaused(false);
        cleanup();
      };
      
      setUtterance(newUtterance);
      speechSynthesis.speak(newUtterance);
      
      // Set a completion check in case onend doesn't fire
      completionCheckRef.current = setTimeout(() => {
        if (!speechSynthesis.speaking) {
          setIsSpeaking(false);
          setIsPaused(false);
          cleanup();
        }
      }, text.length * 50) as unknown as NodeJS.Timeout;
      
    } catch (error) {
      console.error('Failed to initialize TTS:', error);
    }
  };

  const pause = () => {
    if (speechSynthesis && isSpeaking && !isPaused) {
      speechSynthesis.pause();
      setIsPaused(true);
    }
  };

  const resume = () => {
    if (speechSynthesis && isPaused) {
      speechSynthesis.resume();
      setIsPaused(false);
    }
  };

  const stop = () => {
    try {
      if (speechSynthesis) {
        speechSynthesis.cancel();
      }
      setIsSpeaking(false);
      setIsPaused(false);
      setUtterance(null);
      cleanup();
    } catch (error) {
      console.error('Error while stopping TTS:', error);
    }
  };

  // Get available voices
  const getVoices = (): SpeechSynthesisVoice[] => {
    return speechSynthesis?.getVoices() || [];
  };

  return {
    speak,
    pause,
    resume,
    stop,
    getVoices,
    isSpeaking,
    isPaused
  };
};
