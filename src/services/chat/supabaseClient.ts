import { createClient } from "@supabase/supabase-js";

const SUPABASE_URL = "https://agcviwwsvnozhwpkejpv.supabase.co";
const SUPABASE_ANON_KEY =
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImFnY3Zpd3dzdm5vemh3cGtlanB2Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzgxNzQxNjEsImV4cCI6MjA5Mzc1MDE2MX0.AvoEhYrPKQ27WqRqHauYmvShYy8aBk5hOhiQBvIM4GU";

export const supabase = createClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
    realtime: {
        params: {
            eventsPerSecond: 10,
        },
    },
});
