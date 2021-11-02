begin transaction;

create table posts (
	id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
	poster int not null,
	title varchar(255) not null,
	body varchar(512) not null
);

commit transaction;
