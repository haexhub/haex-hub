CREATE TABLE `haex_sync_status` (
	`id` text PRIMARY KEY NOT NULL,
	`backend_id` text NOT NULL,
	`last_pull_sequence` integer,
	`last_push_hlc_timestamp` text,
	`last_sync_at` text,
	`error` text
);
--> statement-breakpoint
ALTER TABLE `haex_extensions` ADD `display_mode` text DEFAULT 'auto';