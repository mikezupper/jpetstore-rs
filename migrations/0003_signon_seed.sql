-- The demo users return. Lesson 3 left signon empty because the original
-- seeds it with plaintext ('j2ee','j2ee') and ('ACID','ACID'); now that the
-- app can hash, the same two users come back as argon2id PHC strings,
-- generated with `cargo run --example mkhash -- <password>`. The passwords
-- are still j2ee and ACID — it's demo data — but the column now holds what
-- a password column should hold, and its contents are useless to a thief.

INSERT INTO signon (username, password) VALUES
    ('j2ee', '$argon2id$v=19$m=19456,t=2,p=1$gojOQRqQjn72GKxmzOqk8w$vCGl+TUFfTDFsSkhmBl3Srbw0xD08wNanYO4qaSgz6Q'),
    ('ACID', '$argon2id$v=19$m=19456,t=2,p=1$uEYU+pq8Uh4DX45kMD60ag$LEt9o/ucS3qIpxSf/X1xPo1GL35QW7QT6WlM0qs0dbA');
