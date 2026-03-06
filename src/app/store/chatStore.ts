import { create } from "zustand";
import { askQuestion } from "../api/tauri";
import type { Citation } from "../types/contracts";

type Message = { role: "user" | "assistant"; text: string; citations?: Citation[] };

type ChatStore = {
  messages: Message[];
  loading: boolean;
  ask: (question: string) => Promise<void>;
};

export const useChatStore = create<ChatStore>((set, get) => ({
  messages: [],
  loading: false,
  ask: async (question) => {
    set({ loading: true, messages: [...get().messages, { role: "user", text: question }] });
    try {
      const res = await askQuestion(question);
      set({
        messages: [...get().messages, { role: "assistant", text: res.answer, citations: res.citations }],
        loading: false
      });
    } catch (error) {
      set({
        messages: [...get().messages, { role: "assistant", text: `Error: ${String(error)}` }],
        loading: false
      });
    }
  }
}));
