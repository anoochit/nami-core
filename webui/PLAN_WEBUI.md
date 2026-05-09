# Plan for ADK-Rust Backend Integration

This plan outlines the steps to integrate the `webui` React frontend with the `adk-server` Rust backend.

## 1. Authentication & Session Management
- **Initialize Session:** On application load, perform a `POST /api/sessions` request to establish a new session. Store the `session_id`.
- **Session Persistence:** Implement logic to handle session retrieval if a `session_id` is already present (e.g., in `localStorage` or `sessionStorage`).

## 2. Chat Interaction
- **Message Sending:** Use the `POST /api/ui/message` endpoint to send user input to the agent.
  - Required JSON body: `{ "session_id": "...", "message": "..." }`
- **Response Handling:** Update the message thread UI with the agent's response received from the API.

## 3. UI/Protocol Integration
- **Capabilities Check:** Use `GET /api/ui/capabilities` to ensure the frontend and backend versions are compatible.
- **Resource Management:** Implement polling with `POST /api/ui/notifications/poll` to handle asynchronous events or notifications from the backend.

## 4. Error Handling
- **API Errors:** Implement centralized error handling for fetch operations.
- **Health Check:** Implement a check using `GET /api/health` before initiating chat interactions to verify backend readiness.

## 5. Implementation Steps
1. Create a `services/api.ts` file for centralized API interactions.
2. Update `App.tsx` to use these API service functions instead of hardcoded `fetch` calls.
3. Integrate real-time notification polling for a more robust interaction.
4. Implement UI loading states and error notifications using `shadcn/ui` components (e.g., `Toast`).
