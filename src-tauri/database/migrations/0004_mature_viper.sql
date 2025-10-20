CREATE TABLE `haex_workspaces` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`position` integer DEFAULT 0 NOT NULL,
	`created_at` integer NOT NULL,
	`haex_tombstone` integer,
	`haex_timestamp` text
);
--> statement-breakpoint
ALTER TABLE `haex_desktop_items` ADD `workspace_id` text NOT NULL REFERENCES haex_workspaces(id);