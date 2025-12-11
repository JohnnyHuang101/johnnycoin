import { useState, useEffect } from 'react';
import { RefreshCw, ArrowUpRight, ArrowDownLeft, Wallet, Activity, LogOut } from 'lucide-react';
import './style.css';
import ReactDOM from 'react-dom/client';
import React from 'react'; // Not strictly needed if using new JSX transform, but good practice



const API_URL = "http://localhost:3000";

function App() {
  // State
  const [user, setUser] = useState(localStorage.getItem('hft_user'));
  const [balance, setBalance] = useState(null);
  
  // Form Inputs
  const [symbolId, setSymbolId] = useState(1);
  const [amount, setAmount] = useState(10);
  const [usernameInput, setUsernameInput] = useState("");
  const [statusMsg, setStatusMsg] = useState("");

  const fetchBalance = async () => {
    if (!user) return;
    try {
      const res = await fetch(`${API_URL}/balance/${user}`);
      const data = await res.json();
      if (!data.error) setBalance(data);
    } catch (e) {
      console.error("Failed to fetch balance", e);
    }
  };

  // Poll for updates every 2 seconds
  useEffect(() => {
    if (user) fetchBalance();
    const interval = setInterval(fetchBalance, 2000); 
    return () => clearInterval(interval);
  }, [user]);

  const handleAuth = async (isLogin) => {
    const endpoint = isLogin ? "/login" : "/register";
    try {
      const res = await fetch(`${API_URL}${endpoint}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username: usernameInput, password: "password", email: "" }),
      });
      const data = await res.json();
      
      if (data.error) {
        setStatusMsg(data.error);
      } else {
        setUser(usernameInput);
        localStorage.setItem('hft_user', usernameInput);
        setStatusMsg(`Success! ID: ${data.user_id}`);
      }
    } catch (e) {
      setStatusMsg("Connection failed");
    }
  };

  const executeTrade = async (action) => {
    if (!user) return;

    let payload = {
      username: user,
      symbol_id: Number(symbolId),
      amount: Number(amount),
      is_cash: false
    };

    // Map UI actions to backend logic
    switch (action) {
      case 'BUY':
        payload.is_cash = false;
        payload.amount = Math.abs(amount); 
        break;
      case 'SELL':
        payload.is_cash = false;
        payload.amount = -Math.abs(amount);
        break;
      case 'DEPOSIT':
        payload.is_cash = true;
        payload.amount = Math.abs(amount);
        break;
      case 'WITHDRAW':
        payload.is_cash = true;
        payload.amount = -Math.abs(amount);
        break;
    }

    try {
      const res = await fetch(`${API_URL}/trade`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      setStatusMsg(JSON.stringify(data));
      fetchBalance(); // Refresh immediately
    } catch (e) {
      setStatusMsg("Trade failed");
    }
  };

  const logout = () => {
    setUser(null);
    localStorage.removeItem('hft_user');
    setBalance(null);
  };

  return (
    <div className="min-h-screen bg-terminal-black text-terminal-text font-mono p-8 selection:bg-terminal-green selection:text-black">
      <div className="max-w-4xl mx-auto space-y-8">
        
        {/* Header */}
        <div className="flex justify-between items-center border-b border-zinc-800 pb-4">
          <h1 className="text-2xl font-bold flex items-center gap-2 text-terminal-green">
            <Activity /> HFT::ENGINE_V1
          </h1>
          {user && (
            <button onClick={logout} className="text-xs hover:text-red-500 flex items-center gap-1">
              LOGOUT <LogOut size={14}/>
            </button>
          )}
        </div>

        {/* Auth Screen */}
        {!user ? (
          <div className="max-w-sm mx-auto bg-terminal-dark p-6 rounded border border-zinc-800 shadow-xl">
            <h2 className="text-lg mb-4 text-center">ACCESS TERMINAL</h2>
            <input 
              className="w-full bg-black border border-zinc-700 p-2 mb-4 text-center focus:outline-none focus:border-terminal-green text-white"
              placeholder="USERNAME"
              value={usernameInput}
              onChange={e => setUsernameInput(e.target.value)}
            />
            <div className="flex gap-2">
              <button onClick={() => handleAuth(true)} className="flex-1 bg-zinc-800 hover:bg-zinc-700 py-2 text-sm text-white">LOGIN</button>
              <button onClick={() => handleAuth(false)} className="flex-1 bg-zinc-800 hover:bg-zinc-700 py-2 text-sm text-white">REGISTER</button>
            </div>
            {statusMsg && <p className="text-center text-xs mt-4 text-red-400">{statusMsg}</p>}
          </div>
        ) : (
          /* Dashboard */
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            
            {/* Left Col: Balance */}
            <div className="space-y-6">
              <div className="bg-terminal-dark p-6 rounded border border-zinc-800 relative">
                <button onClick={fetchBalance} className="absolute top-4 right-4 text-zinc-500 hover:text-white">
                  <RefreshCw size={16} />
                </button>
                <h3 className="text-zinc-500 text-sm mb-1 uppercase">Available Cash</h3>
                <div className="text-4xl font-bold text-terminal-green">
                  ${balance?.cash?.toLocaleString() ?? "0.00"}
                </div>
              </div>

              <div className="bg-terminal-dark p-6 rounded border border-zinc-800">
                <h3 className="text-zinc-500 text-sm mb-4 uppercase flex items-center gap-2">
                  <Wallet size={16}/> Portfolio Holdings
                </h3>
                {balance && balance.stocks && Object.keys(balance.stocks).length > 0 ? (
                  <ul className="space-y-2">
                    {Object.entries(balance.stocks).map(([id, qty]) => (
                      <li key={id} className="flex justify-between items-center bg-black/50 p-2 rounded border border-zinc-900">
                        <span className="text-zinc-400">SYM_ID::{id}</span>
                        <span className="font-bold text-blue-400">{qty} UNITS</span>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="text-zinc-600 italic">No open positions.</p>
                )}
              </div>
            </div>

            {/* Right Col: Execution */}
            <div className="bg-terminal-dark p-6 rounded border border-zinc-800 h-fit">
              <h3 className="text-zinc-500 text-sm mb-4 uppercase">Order Execution</h3>
              
              <div className="space-y-4">
                <div>
                  <label className="block text-xs text-zinc-500 mb-1">SYMBOL ID</label>
                  <input 
                    type="number" 
                    value={symbolId}
                    onChange={e => setSymbolId(e.target.value)}
                    className="w-full bg-black border border-zinc-700 p-2 text-white focus:border-terminal-green outline-none"
                  />
                </div>
                <div>
                  <label className="block text-xs text-zinc-500 mb-1">QUANTITY / AMOUNT</label>
                  <input 
                    type="number" 
                    value={amount}
                    onChange={e => setAmount(e.target.value)}
                    className="w-full bg-black border border-zinc-700 p-2 text-white focus:border-terminal-green outline-none"
                  />
                </div>

                <div className="grid grid-cols-2 gap-2 pt-4">
                  <button 
                    onClick={() => executeTrade('BUY')}
                    className="bg-terminal-green/10 border border-terminal-green text-terminal-green hover:bg-terminal-green hover:text-black py-3 flex justify-center items-center gap-2"
                  >
                    <ArrowUpRight size={16}/> BUY STOCK
                  </button>
                  <button 
                    onClick={() => executeTrade('SELL')}
                    className="bg-red-500/10 border border-red-500 text-red-500 hover:bg-red-500 hover:text-black py-3 flex justify-center items-center gap-2"
                  >
                    <ArrowDownLeft size={16}/> SELL STOCK
                  </button>
                  <button 
                    onClick={() => executeTrade('DEPOSIT')}
                    className="col-span-1 bg-zinc-800 hover:bg-zinc-700 py-2 text-xs text-zinc-400"
                  >
                    DEPOSIT CASH
                  </button>
                  <button 
                    onClick={() => executeTrade('WITHDRAW')}
                    className="col-span-1 bg-zinc-800 hover:bg-zinc-700 py-2 text-xs text-zinc-400"
                  >
                    WITHDRAW CASH
                  </button>
                </div>

                {statusMsg && (
                   <div className="mt-4 p-2 bg-black border border-zinc-800 font-mono text-xs text-zinc-400 break-all">
                     {">"} {statusMsg}
                   </div>
                )}
              </div>
            </div>

          </div>
        )}
      </div>
    </div>
  );
}

export default App;

// 1. Find the root DOM element defined in index.html
const rootElement = document.getElementById('app');

// 2. Create the React root and render the App component inside it
ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);