update reports
set is_banned = true
where id = 4
returning id, user_id;