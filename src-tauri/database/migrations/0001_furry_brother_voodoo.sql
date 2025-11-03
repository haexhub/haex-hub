PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_haex_workspaces` (
	`id` text PRIMARY KEY NOT NULL,
	`device_id` text NOT NULL,
	`name` text NOT NULL,
	`position` integer DEFAULT 0 NOT NULL,
	`background` text,
	`haex_timestamp` text
);
--> statement-breakpoint
INSERT INTO `__new_haex_workspaces`("id", "device_id", "name", "position", "background", "haex_timestamp") SELECT "id", "device_id", "name", "position", "background", "haex_timestamp" FROM `haex_workspaces`;--> statement-breakpoint
DROP TABLE `haex_workspaces`;--> statement-breakpoint
ALTER TABLE `__new_haex_workspaces` RENAME TO `haex_workspaces`;--> statement-breakpoint
PRAGMA foreign_keys=ON;--> statement-breakpoint
CREATE UNIQUE INDEX `haex_workspaces_position_unique` ON `haex_workspaces` (`position`);