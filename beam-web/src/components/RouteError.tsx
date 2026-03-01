import { useRouter } from "@tanstack/react-router";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";

export function RouteError({
	error,
	reset,
}: {
	error: Error;
	reset: () => void;
}) {
	const router = useRouter();

	const handleRetry = () => {
		reset();
		router.invalidate();
	};

	return (
		<div className="min-h-screen bg-gradient-to-br from-gray-950 via-gray-900 to-gray-950 flex items-center justify-center">
			<div className="text-center space-y-4">
				<AlertTriangle className="mx-auto text-amber-400" size={40} />
				<h2 className="text-xl font-semibold text-white">
					Something went wrong
				</h2>
				{import.meta.env.DEV && (
					<p className="text-red-400 text-sm max-w-md mx-auto font-mono bg-gray-900/60 rounded p-3">
						{error.message}
					</p>
				)}
				<Button
					onClick={handleRetry}
					variant="outline"
					className="border-gray-600 text-gray-300 hover:bg-gray-800"
				>
					<RefreshCw size={16} className="mr-2" />
					Try again
				</Button>
			</div>
		</div>
	);
}
