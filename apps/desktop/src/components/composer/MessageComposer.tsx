import { useState, useRef, useEffect } from 'react';
import { Send, Paperclip, AlertCircle } from 'lucide-react';
import './MessageComposer.css';

export function MessageComposer() {
  const [text, setText] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-resize textarea
  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 200) + 'px';
  }, [text]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      // Send will be implemented when providers are connected
    }
  };

  return (
    <div className="composer">
      <div className={`composer-box ${text.trim() ? 'has-text' : ''}`}>
        <button className="composer-action attach" title="Attach file (coming soon)">
          <Paperclip size={18} />
        </button>

        <textarea
          ref={textareaRef}
          placeholder="Message AI-Hub…"
          value={text}
          onChange={e => setText(e.target.value)}
          onKeyDown={handleKeyDown}
          rows={1}
        />

        <button
          className={`composer-action send ${text.trim() ? 'ready' : ''}`}
          disabled={!text.trim()}
          title={text.trim() ? 'Send message (provider not connected)' : 'Type a message'}
        >
          <Send size={18} />
        </button>
      </div>

      <div className="composer-status">
        <AlertCircle size={12} />
        <span>Provider not connected — messages cannot be sent yet</span>
      </div>
    </div>
  );
}
