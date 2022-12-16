// Create a trait that includes a function, executed in a separate thread, that receives data from the device and sends it to the relative service
// The function should return when there's a connection error
// The function needs to be called on an already existing object, that holds configuration settings
// The function should be called from the main thread, and should return a tokio task
// This trait basically is only a function that from a self and a receiver channel, returns a tokio task

use crate::utils::SendData;
use crossbeam_channel::Receiver;
use tokio::task::JoinHandle;

trait ReceiverService {
    fn receive(self, rx: Receiver<SendData>) -> JoinHandle<()>;
}
