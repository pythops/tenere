
/**
 * Safely cancels speech synthesis and handles any potential errors
 */
export const safeCancel = () => {
  try {
    if (window.speechSynthesis) {
      window.speechSynthesis.cancel();
    }
  } catch (error) {
    console.error('Error while cancelling speech synthesis:', error);
  }
};

/**
 * Checks if the browser supports speech synthesis
 */
export const isTTSSupported = (): boolean => {
  return 'speechSynthesis' in window && 
         'SpeechSynthesisUtterance' in window;
};

/**
 * Chunks text into smaller pieces to prevent TTS cutoff
 * @param text The text to chunk
 * @param maxLength Maximum length of each chunk
 * @returns Array of text chunks
 */
export const chunkText = (text: string, maxLength: number = 200): string[] => {
  if (!text || text.length <= maxLength) {
    return [text];
  }

  const chunks: string[] = [];
  let currentChunk = '';
  
  // Split by sentences to create more natural chunks
  const sentences = text.split(/(?<=[.!?])\s+/);
  
  for (const sentence of sentences) {
    if ((currentChunk + sentence).length <= maxLength) {
      currentChunk += (currentChunk ? ' ' : '') + sentence;
    } else {
      // If a single sentence is too long, split by words
      if (currentChunk) {
        chunks.push(currentChunk);
        currentChunk = sentence;
      } else {
        const words = sentence.split(' ');
        currentChunk = words[0];
        
        for (let i = 1; i < words.length; i++) {
          if ((currentChunk + ' ' + words[i]).length <= maxLength) {
            currentChunk += ' ' + words[i];
          } else {
            chunks.push(currentChunk);
            currentChunk = words[i];
          }
        }
      }
    }
  }
  
  if (currentChunk) {
    chunks.push(currentChunk);
  }
  
  return chunks;
};

/**
 * Creates a resilient utterance with proper error handling
 * @param text Text to speak
 * @param options Speech options
 * @param onEnd Callback when speech ends
 * @returns SpeechSynthesisUtterance instance
 */
export const createResilientUtterance = (
  text: string, 
  options?: { rate?: number; pitch?: number; voice?: SpeechSynthesisVoice },
  onEnd?: () => void
): SpeechSynthesisUtterance => {
  const utterance = new SpeechSynthesisUtterance(text);
  
  if (options?.rate) utterance.rate = options.rate;
  if (options?.pitch) utterance.pitch = options.pitch;
  if (options?.voice) utterance.voice = options.voice;
  
  utterance.onend = () => {
    if (onEnd) onEnd();
  };
  
  utterance.onerror = (event) => {
    console.error('TTS error:', event);
    if (onEnd) onEnd();
  };
  
  return utterance;
};
