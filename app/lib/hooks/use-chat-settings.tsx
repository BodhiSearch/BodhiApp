import {
  Dispatch,
  SetStateAction,
  createContext,
  useContext,
  useEffect,
  useState
} from "react";
import { useLocalStorage } from "@/lib/hooks/use-local-storage";
import { getModels } from "../backend";

interface ChatSettingsContext {
  model: string | null
  models: Model[] | null
  setModel: (value: string | null) => void
}

const ChatSettingsContext = createContext<ChatSettingsContext | undefined>(undefined);

export function useChatSettings() {
  const context = useContext(ChatSettingsContext);
  if (!context) {
    throw new Error("useChatSettings must be used within a ChatSettingsProvider");
  }
  return context;
}

interface ChatSettingsProviderProps {
  children: React.ReactNode
}

interface Model {
  alias: string,
}

export function ChatSettingsProvider({ children }: ChatSettingsProviderProps) {
  const [model, setModel] = useLocalStorage<string | null>('model', null);
  const [models, setModels] = useState<Model[] | null>(null);

  const populateModel = async () => {
    let { data, status } = await getModels();
    if (status === 200) {
      setModels(data)
    } else {
      console.log(`error fetching models: [${status}]: ${data}`);
    }
  }

  useEffect(() => {
    populateModel()
  }, [setModels]);

  return <ChatSettingsContext.Provider value={{ model, models, setModel }}>
    {children}
  </ChatSettingsContext.Provider>
}
