import { useState } from "react";
import { useChatStore } from "../store/chatStore";
import { SourcesPanel } from "./SourcesPanel";

export function ChatPanel() {
  const [question, setQuestion] = useState("");
  const { messages, ask, loading } = useChatStore();

  return (
    <div className="panel chat-panel">
      <div className="chat-header">
        <h2>Document Chat</h2>
        <p>Answers grounded in your indexed files.</p>
      </div>
      <div className="chat-window">
        {messages.length === 0 ? (
          <div className="empty-chat">Ask a question after scanning your folder.</div>
        ) : null}
        {messages.map((m, i) => (
          <div key={i} className={`msg ${m.role}`}>
            <strong>{m.role}</strong>
            <div className="msg-text">{m.text}</div>
            {m.role === "assistant" && m.citations ? <SourcesPanel citations={m.citations} /> : null}
          </div>
        ))}
      </div>
      <div className="chat-input-row">
        <input
          style={{ flex: 1 }}
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          placeholder="Ask from indexed docs..."
        />
        <button
          className="btn btn-primary"
          disabled={loading || !question.trim()}
          onClick={() => {
            const q = question.trim();
            setQuestion("");
            void ask(q);
          }}
        >
          Ask
        </button>
      </div>
    </div>
  );
}
