

from typing import Tuple, List, Union
import asyncio
from functools import partial
import commune as c
import aiohttp
import json

import My_module # import rust module



from aiohttp.streams import StreamReader


class Client(c.module('client.http')):

    def __init__( 
            self,
            ip: str ='0.0.0.0',
            port: int = 50053 ,
            network: bool = None,
            key : str = None,
            save_history: bool = True, 
            history_path : str = 'history',
            loop: 'asyncio.EventLoop' = None, 
            debug: bool = False,
            serializer= 'serializer',
            **kwargs
        ):
        self.loop = c.get_event_loop() if loop == None else loop
        self.set_client(ip =ip,port = port)
        self.serializer = c.module(serializer)()
        self.key = c.get_key(key)
        self.my_ip = c.ip()
        self.network = c.resolve_network(network)
        self.start_timestamp = c.timestamp()
        self.save_history = save_history  
        self.history_path = history_path
        self.debug = debug

        

    
    def age(self):
        return  self.start_timestamp - c.timestamp()

    def set_client(self,
            ip: str =None,
            port: int = None ,
            verbose: bool = False 
            ):
        self.ip = ip if ip else c.default_ip
        self.port = port if port else c.free_port() 
        if verbose:
            c.print(f"Connecting to {self.ip}:{self.port}", color='green')
        self.address = f"{self.ip}:{self.port}" 
       

    def resolve_client(self, ip: str = None, port: int = None) -> None:
        if ip != None or port != None:
            self.set_client(ip =ip,port = port)
    


    async def async_forward(self,
        fn: str,
        args: list = None,
        kwargs: dict = None,
        ip: str = None, 
        port : int= None,
        timeout: int = 10,
        generator: bool = False,
        headers : dict ={'Content-Type': 'application/json'},
        ):
        self.resolve_client(ip=ip, port=port)
        args = args if args else []
        kwargs = kwargs if kwargs else {}
        url = f"http://{self.address}/{fn}/" 
        input =  { 
                        "args": args,
                        "kwargs": kwargs,
                        "ip": self.my_ip,
                        "timestamp": c.timestamp(),
                        }
        # serialize this into a json string
        request = self.serializer.serialize(input) 
        request = self.key.sign(request, return_json=True)

        
        
        # start a client session and send the request
        async with aiohttp.ClientSession() as session:

            
            result = My_module.fetch_data(url, json=request, headers=headers)  # fetch_data in rust module


        if isinstance(result, dict):
            result = self.serializer.deserialize(result)
        elif isinstance(result, str):
            result = self.serializer.deserialize(result)
        if isinstance(result, dict) and 'data' in result:
            result = result['data']
        if self.save_history:
            input['fn'] = fn
            input['result'] = result
            input['module']  = self.address
            input['latency'] =  c.time() - input['timestamp']
            path = self.history_path+'/' + self.server_name + '/' + str(input['timestamp'])
            self.put(path, input)
        return result
    
    @classmethod
    def history(cls, key=None, history_path='history'):
        key = c.get_key(key)
        return cls.ls(history_path + '/' + key.ss58_address)
    @classmethod
    def all_history(cls, key=None, history_path='history'):
        key = c.get_key(key)
        return cls.glob(history_path)
        


    @classmethod
    def rm_key_history(cls, key=None, history_path='history'):
        key = c.get_key(key)
        return cls.rm(history_path + '/' + key.ss58_address)
    
    @classmethod
    def rm_history(cls, key=None, history_path='history'):
        key = c.get_key(key)
        return cls.rm(history_path)    


    def process_output(self, result):
        ## handles 
        if isinstance(result, str):
            result = json.loads(result)
        if 'data' in result:
            result = self.serializer.deserialize(result)
            return result['data'] 
        else:
            return result
        
    def forward(self,*args,return_future:bool=False, timeout:str=4, **kwargs):
        forward_future = asyncio.wait_for(self.async_forward(*args, **kwargs), timeout=timeout)
        if return_future:
            return forward_future
        else:
            return self.loop.run_until_complete(forward_future)
        
        
    __call__ = forward

    def __str__ ( self ):
        return "Client({})".format(self.address) 
    def __repr__ ( self ):
        return self.__str__()
    def __exit__ ( self ):
        self.__del__()


    def virtual(self):
        return c.virtual_client(module = self)
    
    def __repr__(self) -> str:
        return super().__repr__()
