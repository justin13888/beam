import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/profile")({
  component: () => <div>Hello /profile!</div>,
}); // TODO: Implement
