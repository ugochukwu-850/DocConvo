
import { useEffect, useRef, useState } from "react";
import "./App.css";
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from "@tauri-apps/api/core";
import {Bot, CircleUser} from "lucide-react";
import Markdown from 'react-markdown'

interface Message {
  person: "user" | "system",
  message: string
}

function App() {
  const [dirPaths, setDirPaths] = useState<undefined | string[]>();
  const [messages, setMessages] = useState<undefined | Message[]>();
  const inputComp = useRef<HTMLInputElement | null>(null);
  const messageContainer = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    messageContainer?.current?.lastElementChild?.scrollIntoView({ behavior: "smooth" });
  }, [messages])


  const addFilePath = async () => {
    let inputdirPaths = (await open({
      multiple: true,
      directory: true,
    }))?.filter(x => !dirPaths?.includes(x));

    // invoke backend command to load the files into vector database
    if (inputdirPaths) {
      try {
        dirPaths ? setDirPaths([...dirPaths, ...inputdirPaths]) : setDirPaths(inputdirPaths);

        await invoke("index_folders", { "folders": inputdirPaths })
      } catch (error) {
        dirPaths?.pop();
      }

    }
  }

  const handleMessage = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    let text = inputComp?.current?.value.trim();
    let innermessages: Message[] = messages ? [...messages!, ...[{ person: "user", message: text } as Message]] : [{ person: "user", message: text } as Message];
    setMessages(innermessages)
    if (inputComp.current) {
      inputComp.current.value = "";
    }
    // invoke the command to send message
    let response: string = await invoke("prompt", { prompt: text });

    // update the messages with the response
    setMessages(messages => [...messages!, ...[{ message: response, person: "system" } as Message]]);

  }

  return (
    <div className="grid grid-rows-12 w-full h-[100vh] dark:bg-neutral-950 dark:text-white p-2 " onLoad={() => { }}>
      <div className="top row-span-1 w-full h-fit py-2 dark:bg-neutral-900 rounded-full grid grid-cols-12">
        <div className="h-fit flex flex-row  dirtags col-span-9 px-1 my-auto overflow-x-scroll" >
          {
            dirPaths?.map(e => {
              return <span className="text-sm text-gray-300 my-auto px-3 p-2 bg-neutral-800 rounded-full mx-1 cursor-pointer">{e.split("/").pop()}</span>
            })
          }
        </div>
        <div className="col-span-3 flex flex-row-reverse px-2">
          <button className="rounded-full bg-purple-950 px-3 py-2 h-fit my-auto text-purple-200 text-sm " onClick={addFilePath}>Add Folder</button>
        </div>
      </div>
      <div className="center h-full row-span-10 overflow-scroll flex-col" ref={messageContainer} >
        {
          messages?.map(message => {
            return <div className={`px-4 py-2 mx-auto m-2  w-fit flex flex-row ${message.person == "user" ? "mr-0" : "ml-0"}`}>
              {message.person == "system" ? <Bot className="min-w-5" /> : <CircleUser/>}              
              <p className="text-gray-400 px-2"><Markdown>{message.message}</Markdown></p>
            </div>
          })
        }
      </div>
      <form onSubmit={handleMessage} className="buttom row-span-2 my-auto px-2">
        <input ref={inputComp} type="text" placeholder="Start your conversation" className="outline-1 w-full rounded-full outline-neutral-600 shadow-sm shadow-nuetral-300 p-3 px-6 text-sm " />
      </form>
    </div>
  )
}

export default App;
