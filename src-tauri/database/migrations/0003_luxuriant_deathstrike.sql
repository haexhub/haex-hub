CREATE TABLE `haex_sync_backends` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`server_url` text NOT NULL,
	`enabled` integer DEFAULT true NOT NULL,
	`priority` integer DEFAULT 0 NOT NULL,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`updated_at` integer,
	`haex_timestamp` text
);
