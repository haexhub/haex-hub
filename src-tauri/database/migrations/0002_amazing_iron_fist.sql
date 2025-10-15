PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_haex_extensions` (
	`id` text PRIMARY KEY NOT NULL,
	`public_key` text NOT NULL,
	`name` text NOT NULL,
	`version` text NOT NULL,
	`author` text,
	`description` text,
	`entry` text DEFAULT 'index.html' NOT NULL,
	`homepage` text,
	`enabled` integer DEFAULT true,
	`icon` text,
	`signature` text NOT NULL,
	`haex_tombstone` integer,
	`haex_timestamp` text
);
--> statement-breakpoint
INSERT INTO `__new_haex_extensions`("id", "public_key", "name", "version", "author", "description", "entry", "homepage", "enabled", "icon", "signature", "haex_tombstone", "haex_timestamp") SELECT "id", "public_key", "name", "version", "author", "description", "entry", "homepage", "enabled", "icon", "signature", "haex_tombstone", "haex_timestamp" FROM `haex_extensions`;--> statement-breakpoint
DROP TABLE `haex_extensions`;--> statement-breakpoint
ALTER TABLE `__new_haex_extensions` RENAME TO `haex_extensions`;--> statement-breakpoint
PRAGMA foreign_keys=ON;--> statement-breakpoint
CREATE UNIQUE INDEX `haex_extensions_public_key_name_unique` ON `haex_extensions` (`public_key`,`name`);