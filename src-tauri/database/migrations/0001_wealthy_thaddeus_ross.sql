CREATE TABLE `testTable` (
	`id` text PRIMARY KEY NOT NULL,
	`author` text,
	`test` text
);
--> statement-breakpoint
ALTER TABLE `haex_extensions` ADD `icon` text;