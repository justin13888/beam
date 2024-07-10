import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/admin/dashboard")({
  component: () => <div>Hello /dashboard!</div>,
}); // TODO: Implement admin dashboard and subroutes
