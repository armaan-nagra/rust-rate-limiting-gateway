#!/bin/bash

echo "⚡ Quick Rate Limiter Test"
echo "========================"

SERVER_URL="http://127.0.0.1:3000"

# Get token
echo "Getting token..."
TOKEN=$(curl -s -X POST "$SERVER_URL/register" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to get token. Start server with: cargo run"
    exit 1
fi

echo "Token: $TOKEN"
echo "Making 65 rapid requests..."

SUCCESS=0
BLOCKED=0

for i in {1..65}; do
    HTTP_CODE=$(curl -s -w "%{http_code}" -o /dev/null -H "Authorization: Bearer $TOKEN" "$SERVER_URL/api/data")
    
    if [ "$HTTP_CODE" == "200" ]; then
        SUCCESS=$((SUCCESS + 1))
        echo -n "✅"
    elif [ "$HTTP_CODE" == "429" ]; then
        BLOCKED=$((BLOCKED + 1))
        echo -n "🚫"
    else
        echo -n "❓"
    fi
    
    # Print newline every 10 requests for readability
    if [ $((i % 10)) -eq 0 ]; then
        echo " ($i)"
    fi
done

echo ""
echo "Results: $SUCCESS successful, $BLOCKED blocked"

if [ $SUCCESS -le 60 ] && [ $BLOCKED -gt 0 ]; then
    echo "Rate limiter working!"
else
    echo "Unexpected results"
fi 