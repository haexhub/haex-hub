CREATE TABLE `haex_desktop_items` (
	`id` text PRIMARY KEY NOT NULL,
	`item_type` text NOT NULL,
	`reference_id` text NOT NULL,
	`position_x` integer DEFAULT 0 NOT NULL,
	`position_y` integer DEFAULT 0 NOT NULL,
	`haex_tombstone` integer,
	`haex_timestamp` text
);
