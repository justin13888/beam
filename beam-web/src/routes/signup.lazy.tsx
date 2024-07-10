import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/signup")({
  component: () => <div>Hello /signup!</div>,
}); // TODO: Implement
