import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/watch")({
  component: () => <div>Hello /watch!</div>,
}); // TODO: Implement video player
