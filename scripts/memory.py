#!/usr/bin/env python3
"""
Argus Memory Bridge
Handles persistent memory via Supabase (if configured) or local SQLite fallback.
Called from Rust via shell for memory operations.

Usage:
    python memory.py remember '{"content": "...", "type": "fact", "importance": 7}'
    python memory.py recall '{"query": "coffee", "limit": 10}'
    python memory.py forget '{"content_match": "old stuff"}'
    python memory.py list '{}'
"""

import json
import os
import sys
import sqlite3
from pathlib import Path
from datetime import datetime

ARGUS_DIR = Path.home() / ".argus"
DB_PATH = ARGUS_DIR / "memory.db"

# Check for Supabase config
SUPABASE_URL = None
SUPABASE_KEY = None

def load_supabase_config():
    """Try to load Supabase creds from vault or env"""
    global SUPABASE_URL, SUPABASE_KEY
    
    # Check environment first
    SUPABASE_URL = os.environ.get("ARGUS_SUPABASE_URL")
    SUPABASE_KEY = os.environ.get("ARGUS_SUPABASE_KEY")
    
    if SUPABASE_URL and SUPABASE_KEY:
        return True
    
    # Could also check a config file here
    config_file = ARGUS_DIR / "supabase.json"
    if config_file.exists():
        try:
            with open(config_file) as f:
                config = json.load(f)
                SUPABASE_URL = config.get("url")
                SUPABASE_KEY = config.get("key")
                return bool(SUPABASE_URL and SUPABASE_KEY)
        except:
            pass
    
    return False

def get_supabase():
    """Get Supabase client if available"""
    if not SUPABASE_URL or not SUPABASE_KEY:
        return None
    try:
        from supabase import create_client
        return create_client(SUPABASE_URL, SUPABASE_KEY)
    except ImportError:
        return None

def init_sqlite():
    """Initialize local SQLite database"""
    ARGUS_DIR.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(DB_PATH)
    conn.execute("""
        CREATE TABLE IF NOT EXISTS memories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type TEXT NOT NULL,
            content TEXT NOT NULL,
            reasoning TEXT,
            importance REAL DEFAULT 5.0,
            tags TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
    """)
    conn.commit()
    return conn

# ============ OPERATIONS ============

def remember(args: dict) -> dict:
    """Store a memory"""
    content = args.get("content", "")
    mem_type = args.get("type", "fact")
    importance = args.get("importance", 5.0)
    reasoning = args.get("reasoning")
    tags = args.get("tags")
    
    if not content:
        return {"success": False, "error": "No content provided"}
    
    sb = get_supabase()
    if sb:
        try:
            sb.table("argus_memories").insert({
                "type": mem_type,
                "content": content,
                "reasoning": reasoning,
                "importance": importance,
                "tags": tags
            }).execute()
            return {"success": True, "message": f"Remembered: {content[:50]}...", "backend": "supabase"}
        except Exception as e:
            # Fall through to SQLite
            pass
    
    # SQLite fallback
    conn = init_sqlite()
    conn.execute(
        "INSERT INTO memories (type, content, reasoning, importance, tags) VALUES (?, ?, ?, ?, ?)",
        (mem_type, content, reasoning, importance, json.dumps(tags) if tags else None)
    )
    conn.commit()
    conn.close()
    
    return {"success": True, "message": f"Remembered: {content[:50]}...", "backend": "sqlite"}

def recall(args: dict) -> dict:
    """Search and retrieve memories"""
    query = args.get("query")
    mem_type = args.get("type")
    limit = args.get("limit", 10)
    
    sb = get_supabase()
    if sb:
        try:
            q = sb.table("argus_memories").select("*").order("importance", desc=True).limit(limit)
            if mem_type:
                q = q.eq("type", mem_type)
            if query:
                q = q.ilike("content", f"%{query}%")
            result = q.execute()
            
            memories = [{"type": m["type"], "content": m["content"], "importance": m["importance"]} for m in result.data]
            return {"success": True, "memories": memories, "count": len(memories), "backend": "supabase"}
        except Exception as e:
            pass
    
    # SQLite fallback
    conn = init_sqlite()
    sql = "SELECT type, content, importance FROM memories WHERE 1=1"
    params = []
    
    if mem_type:
        sql += " AND type = ?"
        params.append(mem_type)
    if query:
        sql += " AND content LIKE ?"
        params.append(f"%{query}%")
    
    sql += " ORDER BY importance DESC LIMIT ?"
    params.append(limit)
    
    cursor = conn.execute(sql, params)
    memories = [{"type": r[0], "content": r[1], "importance": r[2]} for r in cursor.fetchall()]
    conn.close()
    
    return {"success": True, "memories": memories, "count": len(memories), "backend": "sqlite"}

def forget(args: dict) -> dict:
    """Delete memories matching content"""
    content_match = args.get("content_match", "")
    
    if not content_match:
        return {"success": False, "error": "No content_match provided"}
    
    sb = get_supabase()
    if sb:
        try:
            sb.table("argus_memories").delete().ilike("content", f"%{content_match}%").execute()
            return {"success": True, "message": f"Forgot memories matching: {content_match}", "backend": "supabase"}
        except:
            pass
    
    # SQLite fallback
    conn = init_sqlite()
    cursor = conn.execute("DELETE FROM memories WHERE content LIKE ?", (f"%{content_match}%",))
    deleted = cursor.rowcount
    conn.commit()
    conn.close()
    
    return {"success": True, "message": f"Forgot {deleted} memories matching: {content_match}", "backend": "sqlite"}

def list_all(args: dict) -> dict:
    """List all memories"""
    return recall({"limit": args.get("limit", 50)})

def status(args: dict) -> dict:
    """Check memory system status"""
    sb = get_supabase()
    return {
        "success": True,
        "supabase_configured": bool(SUPABASE_URL and SUPABASE_KEY),
        "supabase_available": sb is not None,
        "sqlite_path": str(DB_PATH),
        "sqlite_exists": DB_PATH.exists()
    }

# ============ MAIN ============

OPERATIONS = {
    "remember": remember,
    "recall": recall,
    "forget": forget,
    "list": list_all,
    "status": status,
}

def main():
    if len(sys.argv) < 2:
        print(json.dumps({"success": False, "error": "Usage: memory.py <operation> [json_args]"}))
        sys.exit(1)
    
    operation = sys.argv[1]
    
    if operation not in OPERATIONS:
        print(json.dumps({"success": False, "error": f"Unknown operation: {operation}"}))
        sys.exit(1)
    
    # Parse args
    args = {}
    if len(sys.argv) > 2:
        try:
            args = json.loads(sys.argv[2])
        except json.JSONDecodeError as e:
            print(json.dumps({"success": False, "error": f"Invalid JSON: {e}"}))
            sys.exit(1)
    
    # Load Supabase config
    load_supabase_config()
    
    # Execute
    try:
        result = OPERATIONS[operation](args)
        print(json.dumps(result))
    except Exception as e:
        print(json.dumps({"success": False, "error": str(e)}))
        sys.exit(1)

if __name__ == "__main__":
    main()
