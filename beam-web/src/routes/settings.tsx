import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/settings")({
  component: () => <div>Hello /settings!</div>,
}); // TODO: Allows users to configure preferences such as playback settings, notification preferences, and account privacy.
