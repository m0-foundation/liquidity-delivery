-----------------------
-- Unified events views
-----------------------

CREATE OR REPLACE VIEW public.events_cancel_reported AS
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash FROM solanadevnet.events_cancel_reported
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash FROM sepolia.events_cancel_reported
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash FROM arbitrumsepolia.events_cancel_reported
  ORDER BY ts DESC;

CREATE OR REPLACE VIEW public.events_fill_reported AS
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, amount_in_to_release, amount_out_filled, origin_recipient FROM solanadevnet.events_fill_reported
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, amount_in_to_release, amount_out_filled, origin_recipient FROM sepolia.events_fill_reported
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, amount_in_to_release, amount_out_filled, origin_recipient FROM arbitrumsepolia.events_fill_reported
  ORDER BY ts DESC;

CREATE OR REPLACE VIEW public.events_order_cancelled AS
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash FROM solanadevnet.events_order_cancelled
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash FROM sepolia.events_order_cancelled
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash FROM arbitrumsepolia.events_order_cancelled
  ORDER BY ts DESC;

CREATE OR REPLACE VIEW public.events_order_completed AS
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash FROM solanadevnet.events_order_completed
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash FROM sepolia.events_order_completed
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash FROM arbitrumsepolia.events_order_completed
  ORDER BY ts DESC;

CREATE OR REPLACE VIEW public.events_order_filled AS
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, solver, amount_in_to_release, amount_out_filled FROM solanadevnet.events_order_filled
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, solver, amount_in_to_release, amount_out_filled FROM sepolia.events_order_filled
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, solver, amount_in_to_release, amount_out_filled FROM arbitrumsepolia.events_order_filled
  ORDER BY ts DESC;

CREATE OR REPLACE VIEW public.events_order_opened AS
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, sender, token_in, amount_in, dest_chain_id, token_out, amount_out, solver FROM solanadevnet.events_order_opened
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, sender, token_in, amount_in, dest_chain_id, token_out, amount_out, solver FROM sepolia.events_order_opened
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, sender, token_in, amount_in, dest_chain_id, token_out, amount_out, solver FROM arbitrumsepolia.events_order_opened
  ORDER BY ts DESC;

CREATE OR REPLACE VIEW public.events_refund_claimed AS
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, sender, amount FROM solanadevnet.events_refund_claimed
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, sender, amount FROM sepolia.events_refund_claimed
  UNION ALL
  SELECT order_id, to_timestamp(ts) AS ts, chain_id, transaction_hash, sender, amount FROM arbitrumsepolia.events_refund_claimed
  ORDER BY ts DESC;

-----------------------
-- Indexes
-----------------------

CREATE INDEX IF NOT EXISTS idx_cancel_reported_order_id   ON sepolia.events_cancel_reported (order_id);
CREATE INDEX IF NOT EXISTS idx_fill_reported_order_id     ON sepolia.events_fill_reported (order_id);
CREATE INDEX IF NOT EXISTS idx_order_cancelled_order_id   ON sepolia.events_order_cancelled (order_id);
CREATE INDEX IF NOT EXISTS idx_order_completed_order_id   ON sepolia.events_order_completed (order_id);
CREATE INDEX IF NOT EXISTS idx_order_filled_order_id      ON sepolia.events_order_filled (order_id);
CREATE INDEX IF NOT EXISTS idx_order_opened_order_id      ON sepolia.events_order_opened (order_id);
CREATE INDEX IF NOT EXISTS idx_refund_claimed_order_id    ON sepolia.events_refund_claimed (order_id);
CREATE INDEX IF NOT EXISTS idx_order_opened_sender        ON sepolia.events_order_opened (sender);
CREATE INDEX IF NOT EXISTS idx_cancel_reported_ts         ON sepolia.events_cancel_reported (ts);
CREATE INDEX IF NOT EXISTS idx_fill_reported_ts           ON sepolia.events_fill_reported (ts);
CREATE INDEX IF NOT EXISTS idx_order_cancelled_ts         ON sepolia.events_order_cancelled (ts);
CREATE INDEX IF NOT EXISTS idx_order_completed_ts         ON sepolia.events_order_completed (ts);
CREATE INDEX IF NOT EXISTS idx_order_filled_ts            ON sepolia.events_order_filled (ts);
CREATE INDEX IF NOT EXISTS idx_order_opened_ts            ON sepolia.events_order_opened (ts);
CREATE INDEX IF NOT EXISTS idx_refund_claimed_ts          ON sepolia.events_refund_claimed (ts);

CREATE INDEX IF NOT EXISTS idx_cancel_reported_order_id   ON solanadevnet.events_cancel_reported (order_id);
CREATE INDEX IF NOT EXISTS idx_fill_reported_order_id     ON solanadevnet.events_fill_reported (order_id);
CREATE INDEX IF NOT EXISTS idx_order_cancelled_order_id   ON solanadevnet.events_order_cancelled (order_id);
CREATE INDEX IF NOT EXISTS idx_order_completed_order_id   ON solanadevnet.events_order_completed (order_id);
CREATE INDEX IF NOT EXISTS idx_order_filled_order_id      ON solanadevnet.events_order_filled (order_id);
CREATE INDEX IF NOT EXISTS idx_order_opened_order_id      ON solanadevnet.events_order_opened (order_id);
CREATE INDEX IF NOT EXISTS idx_refund_claimed_order_id    ON solanadevnet.events_refund_claimed (order_id);
CREATE INDEX IF NOT EXISTS idx_order_opened_sender        ON solanadevnet.events_order_opened (sender);
CREATE INDEX IF NOT EXISTS idx_cancel_reported_ts         ON solanadevnet.events_cancel_reported (ts);
CREATE INDEX IF NOT EXISTS idx_fill_reported_ts           ON solanadevnet.events_fill_reported (ts);
CREATE INDEX IF NOT EXISTS idx_order_cancelled_ts         ON solanadevnet.events_order_cancelled (ts);
CREATE INDEX IF NOT EXISTS idx_order_completed_ts         ON solanadevnet.events_order_completed (ts);
CREATE INDEX IF NOT EXISTS idx_order_filled_ts            ON solanadevnet.events_order_filled (ts);
CREATE INDEX IF NOT EXISTS idx_order_opened_ts            ON solanadevnet.events_order_opened (ts);
CREATE INDEX IF NOT EXISTS idx_refund_claimed_ts          ON solanadevnet.events_refund_claimed (ts);

CREATE INDEX IF NOT EXISTS idx_cancel_reported_order_id   ON arbitrumsepolia.events_cancel_reported (order_id);
CREATE INDEX IF NOT EXISTS idx_fill_reported_order_id     ON arbitrumsepolia.events_fill_reported (order_id);
CREATE INDEX IF NOT EXISTS idx_order_cancelled_order_id   ON arbitrumsepolia.events_order_cancelled (order_id);
CREATE INDEX IF NOT EXISTS idx_order_completed_order_id   ON arbitrumsepolia.events_order_completed (order_id);
CREATE INDEX IF NOT EXISTS idx_order_filled_order_id      ON arbitrumsepolia.events_order_filled (order_id);
CREATE INDEX IF NOT EXISTS idx_order_opened_order_id      ON arbitrumsepolia.events_order_opened (order_id);
CREATE INDEX IF NOT EXISTS idx_refund_claimed_order_id    ON arbitrumsepolia.events_refund_claimed (order_id);
CREATE INDEX IF NOT EXISTS idx_order_opened_sender        ON arbitrumsepolia.events_order_opened (sender);
CREATE INDEX IF NOT EXISTS idx_cancel_reported_ts         ON arbitrumsepolia.events_cancel_reported (ts);
CREATE INDEX IF NOT EXISTS idx_fill_reported_ts           ON arbitrumsepolia.events_fill_reported (ts);
CREATE INDEX IF NOT EXISTS idx_order_cancelled_ts         ON arbitrumsepolia.events_order_cancelled (ts);
CREATE INDEX IF NOT EXISTS idx_order_completed_ts         ON arbitrumsepolia.events_order_completed (ts);
CREATE INDEX IF NOT EXISTS idx_order_filled_ts            ON arbitrumsepolia.events_order_filled (ts);
CREATE INDEX IF NOT EXISTS idx_order_opened_ts            ON arbitrumsepolia.events_order_opened (ts);
CREATE INDEX IF NOT EXISTS idx_refund_claimed_ts          ON arbitrumsepolia.events_refund_claimed (ts);


-------------------
-- Analytical views
-------------------

-- Full order lifecycle: every order with its current status and fill/completion details
CREATE OR REPLACE VIEW public.orders AS
  SELECT
    o.order_id,
    CASE
      WHEN c.order_id IS NOT NULL THEN 'completed'
      WHEN r.order_id IS NOT NULL THEN 'refunded'
      WHEN x.order_id IS NOT NULL THEN 'cancelled'
      WHEN f.order_id IS NOT NULL THEN 'filled'
      ELSE 'open'
    END AS status,
    o.chain_id AS origin_chain_id,
    o.dest_chain_id,
    o.sender,
    o.token_in,
    o.amount_in,
    o.token_out,
    o.amount_out,
    o.ts AS opened_at,
    o.transaction_hash AS open_tx,
    f.fill_count,
    f.total_amount_out_filled,
    f.total_amount_in_to_release,
    f.first_filled_at,
    f.last_filled_at,
    c.ts AS completed_at,
    c.transaction_hash AS complete_tx,
    x.ts AS cancelled_at,
    x.transaction_hash AS cancel_tx,
    r.amount AS refund_amount,
    r.ts AS refunded_at
  FROM public.events_order_opened o
  LEFT JOIN (
    SELECT
      order_id,
      COUNT(*) AS fill_count,
      SUM(amount_out_filled) AS total_amount_out_filled,
      SUM(amount_in_to_release) AS total_amount_in_to_release,
      MIN(ts) AS first_filled_at,
      MAX(ts) AS last_filled_at
    FROM public.events_order_filled
    GROUP BY order_id
  ) f ON f.order_id = o.order_id
  LEFT JOIN public.events_order_completed c ON c.order_id = o.order_id
  LEFT JOIN public.events_order_cancelled x ON x.order_id = o.order_id
  LEFT JOIN public.events_refund_claimed r ON r.order_id = o.order_id
  ORDER BY o.ts DESC;

-- Solver leaderboard: fill counts and volumes per solver
CREATE OR REPLACE VIEW public.solver_stats AS
  SELECT
    f.solver,
    COUNT(*) AS fills,
    SUM(f.amount_out_filled) AS total_amount_out,
    SUM(f.amount_in_to_release) AS total_amount_in,
    MIN(f.ts) AS first_fill,
    MAX(f.ts) AS last_fill
  FROM public.events_order_filled f
  GROUP BY f.solver
  ORDER BY fills DESC;

-- Volume per route (origin chain + dest chain + token pair)
CREATE OR REPLACE VIEW public.route_stats AS
  SELECT
    o.chain_id AS origin_chain_id,
    o.dest_chain_id,
    o.token_in,
    o.token_out,
    COUNT(*) AS order_count,
    SUM(o.amount_in) AS total_amount_in,
    SUM(o.amount_out) AS total_amount_out,
    COUNT(*) FILTER (WHERE c.order_id IS NOT NULL) AS completed_count,
    COUNT(*) FILTER (WHERE x.order_id IS NOT NULL) AS cancelled_count
  FROM public.events_order_opened o
  LEFT JOIN public.events_order_completed c ON c.order_id = o.order_id
  LEFT JOIN public.events_order_cancelled x ON x.order_id = o.order_id
  GROUP BY o.chain_id, o.dest_chain_id, o.token_in, o.token_out
  ORDER BY order_count DESC;
