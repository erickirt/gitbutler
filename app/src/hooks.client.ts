import { showError } from '$lib/notifications/toasts';
import { captureException } from '@sentry/sveltekit';
import { error as logErrorToFile } from 'tauri-plugin-log-api';
import type { HandleClientError } from '@sveltejs/kit';

// SvelteKit error handler.
export function handleError({
	error,
	status
}: {
	error: unknown;
	status: number;
}): ReturnType<HandleClientError> {
	if (status !== 404) {
		logError(error);
	}
	return {
		message: String(error)
	};
}

// Handler for unhandled errors inside promises.
window.onunhandledrejection = (e: PromiseRejectionEvent) => {
	logError(e.reason);
};

function logError(error: unknown) {
	let message = error instanceof Error ? error.message : String(error);
	const stack = error instanceof Error ? error.stack : undefined;

	const id = captureException(message, {
		mechanism: {
			type: 'sveltekit',
			handled: false
		}
	});
	message = `${id}: ${message}\n`;
	if (stack) message = `${message}\n${stack}\n`;

	logErrorToFile(message);
	console.error(message);
	showError('Something went wrong', message);
	return id;
}
