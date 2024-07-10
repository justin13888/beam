import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/browse")({
  component: () => <div>Hello /browse!</div>,
});
