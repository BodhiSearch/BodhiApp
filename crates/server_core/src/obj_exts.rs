use llamacpp_rs::CommonParams;
use objs::GptContextParams;

pub fn update(slf: &GptContextParams, gpt_params: &mut CommonParams) {
  // gpt_params.n_threads = self.n_threads;
  gpt_params.seed = slf.n_seed;
  gpt_params.n_ctx = slf.n_ctx;
  gpt_params.n_predict = slf.n_predict;
  gpt_params.n_parallel = slf.n_parallel;
  gpt_params.n_keep = slf.n_keep;
}
