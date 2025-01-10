# FIX Points
WITH TotalGbps AS (
    SELECT
        u."UserID",
        COALESCE(SUM(g."Value"), 0) AS TotalGbps
    FROM
        public."Users" u
    LEFT JOIN public."Gbps" g ON g."UserID" = u."UserID"
    GROUP BY u."UserID"
),
TotalBbps AS (
    SELECT
        u."UserID",
        COALESCE(SUM(b."Value"), 0) AS TotalBbps
    FROM
        public."Users" u
    LEFT JOIN public."Bbps" b ON b."UserID" = u."UserID" AND b."Forgiven" = false
    GROUP BY u."UserID"
),
PointsCTE AS (
    SELECT
        u."UserID",
        b.TotalBbps - g.TotalGbps AS Points
    FROM
        public."Users" u
    LEFT JOIN TotalGbps g ON g."UserID" = u."UserID"
    LEFT JOIN TotalBbps b ON b."UserID" = u."UserID"
)
UPDATE public."Users" u
SET "Points" = cte.Points
FROM PointsCTE cte
WHERE u."UserID" = cte."UserID";
