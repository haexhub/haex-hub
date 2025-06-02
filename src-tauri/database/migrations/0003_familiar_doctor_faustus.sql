CREATE TABLE `haex_notofications` (
	`id` text PRIMARY KEY NOT NULL,
	`title` text,
	`text` text,
	`type` text NOT NULL,
	`read` integer,
	`date` text,
	`image` text,
	`alt` text,
	`icon` text
);
