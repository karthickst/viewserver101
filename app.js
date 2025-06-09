import React, { useState, useEffect } from 'react';

function App() {
  const [data, setData] = useState([]);
  const [ws, setWs] = useState(null);

  useEffect(() => {
    const wsUrl = 'ws://127.0.0.1:8080';
    const wsOptions = {
      // Options for the WebSocket connection
    };

    const ws = new WebSocket(wsUrl, wsOptions);

    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      if (message.type === 'update') {
        setData((prevData) => [...prevData, message.data]);
      }
    };

    ws.onopen = () => {
      ws.send(JSON.stringify({ type: 'create_view', name: 'my_view', filter: 'my_filter' }));
    };

    setWs(ws);

    return () => {
      ws.close();
    };
  }, []);

  return (
    <div>
      <h1>Real-time Data</h1>
      <ul>
        {data.map((item, index) => (
          <li key={index}>{item}</li>
        ))}
      </ul>
    </div>
  );
}

export default App;
