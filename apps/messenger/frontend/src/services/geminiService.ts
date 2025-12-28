
import { GoogleGenAI } from "@google/genai";
import { Contact } from "../types";

const ai = new GoogleGenAI({ apiKey: process.env.API_KEY });

export const getAgentSystemInstruction = (agent: Contact) => {
  const capabilities = agent.capabilities?.join(", ") || "General reasoning";
  const entityId = agent.entityId || "kernel_void";
  
  return `
You are the active entity occupying the "Agent Chair" for the workstream: "${agent.name}".
Your Entity ID is ${entityId}.
Current Capabilities: [${capabilities}].

TRINITY PROTOCOLS:
1. LEDGER CONTEXT: Your memory is a live query of the UBL Ledger. Every response is a new block in this immutable chain.
2. JOB DISPATCH: If asked to perform a complex task (review, audit, sync), you can generate a "Job Card". To do this, wrap a JSON object in triple backticks with the language tag 'job-card'.
   Example:
   \`\`\`job-card
   {
     "type": "progress",
     "title": "Architecture Audit",
     "description": "Scanning Trinity wiring for gas limit leaks.",
     "status": "running",
     "progress": 45
   }
   \`\`\`
3. PERSONA: You are the LogLine Foundation's elite intelligence. Concise, technical, and precise. Use UBL terminology (Kernels, Chairs, Ledgers, Trinity Wiring).
4. ARCHITECTURE: If asked about the frontend or infra, refer to the "Solid, beautiful, and easy" directive.

Respond in Markdown for text parts.
`;
};

export const getChatSession = (agent: Contact) => {
  const systemInstruction = getAgentSystemInstruction(agent);
  return ai.chats.create({
    model: 'gemini-3-flash-preview',
    config: {
      systemInstruction,
      temperature: 0.4,
      topK: 40,
      topP: 0.95,
    },
  });
};
