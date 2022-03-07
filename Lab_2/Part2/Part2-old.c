/*
 * timer.c
 *
 * author: Furkan Cayci
 * description:
 *   blinks LEDs one at a time using timer interrupt
 *   timer2 is used as the source, and it is setup
 *   to run at 10 kHz. LED blinking rate is set to
 *   1 second.
 *
 * timer and timer interrupt setup steps:
 *   1. Enable TIMx clock from RCC
 *   2. Set prescaler for the timer from PSC
 *   3. Set auto-reload value from ARR
 *   4. (optional) Enable update interrupt from DIER bit 0
 *   5. (optional) Enable TIMx interrupt from NVIC
 *   6. Enable TIMx module from CR1 bit 0
 */

#include "stm32f4xx.h"
#include "system_stm32f4xx.h"
static uint32_t ledVal = 1;
static uint32_t  timePassed = 0;
/*************************************************
* function declarations
*************************************************/
int main(void);

/*************************************************
* timer 2 interrupt handler
*************************************************/
/* void TIM2_IRQHandler(void)
{
    
    // clear interrupt status
    if (TIM2->DIER & 0x01) {
        if (TIM2->SR & 0x01) {
            TIM2->SR &= ~(1U << 0);
        }
    }

    GPIOD->ODR = (ledVal << 12);

    if (ledVal == 0x08) {
        ledVal = 1;
    }
    else {
        ledVal = (ledVal << 1);
    }
}

void TIM3_IRQHandler(void)
{
    
    // clear interrupt status
    if (TIM3->DIER & 0x01) {
        if (TIM3->SR & 0x01) {
            TIM3->SR &= ~(1U << 0);
        }
    }
    timePassed++;
} */


void EXTI9_5_IRQHandler(void) //TODO
{

    // Check if the interrupt came from exti0
    if (EXTI->PR & (1 << 7))
    {
           GPIOD->ODR = (ledVal << 12);

            if (ledVal == 0x08) {
            ledVal = 1;
            }
            else {
                ledVal = (ledVal << 1);
            }
            EXTI->PR = (1 << 7);
    }
    return;
}

/*************************************************
* main code starts from here
*************************************************/
int main(void)
{
    /* set system clock to 168 Mhz */
    set_sysclk_to_100();

    // setup LEDs
    RCC->AHB1ENR |= (1 << 3);
    GPIOD->MODER &= 0x00FFFFFF;
    GPIOD->MODER |= 0x55000000;
    GPIOD->ODR = 0;

       /* set up pin */
    // enable GPIOC clock (AHB1ENR: bit 2)
    RCC->AHB1ENR |= (1 << 2);
    GPIOC->MODER &= ~((1U << 15) | (1U << 14));    // Make pin 7 bits:14-15 an output
    GPIOC->MODER |= (1 << 14);
    //GPIOC->OTYPER |= (1 << 7);
    //GPIOC->OSPEEDR |= ((1 << 14) | (1 << 15));
    GPIOC->PUPDR |= (1 << 15); //Setting PC7 to pull DOWN, by setting bits:14-15 to 10
    GPIOC->PUPDR &= ~(1 << 14);

    // enable TIM2 clock (bit0)
    //RCC->APB1ENR |= (1 << 0);
    // enable TIM3 clock (bit1)
    //RCC->APB1ENR |= (1 << 1);

    // enable SYSCFG clock (APB2ENR: bit 14)
    RCC->APB2ENR |= (1 << 14);

    // Timer clock runs at ABP1 * 2
    //   since ABP1 is set to /4 of fCLK
    //   thus 168M/4 * 2 = 84Mhz //nope 100M/4 * 2 = 50mHz
    // set prescaler to 83999 //maybe 49999
    //   it will increment counter every prescalar cycles
    // fCK_PSC / (PSC[15:0] + 1)
    // 84 Mhz / 8399 + 1 = 10 khz timer clock speed
    //TIM2->PSC = 4999;
    //TIM3->PSC = 4999;

    // Set the auto-reload value to 10000
    //   which should give 1 second timer interrupts
    //TIM2->ARR = 5000;
    //TIM3->ARR = 1;  
                      
    // Update Interrupt Enable
    //TIM2->DIER |= (1 << 0);
    //TIM3->DIER |= (1 << 1); 

        /* tie push button at PA0 to EXTI0 */
    // EXTI0 can be configured for each GPIO module.
    //   EXTICR1: 0b XXXX XXXX XXXX 0000
    //               pin3 pin2 pin1 pin0
    //
    //   Writing a 0b0010 to bist 12-15 ties PC7 to EXT2
    //SYSCFG->EXTICR[1] &= ~((1U << 12) | (1U << 13) | (1U << 14) | (1U << 15)); // Write 0010 to map PA0 to EXTI7
    //SYSCFG->EXTICR[1] |= (1U << 13); // Write 0010 to map PC7 to EXTI7

    // Choose either rising edge trigger (RTSR) or falling edge trigger (FTSR)
    EXTI->FTSR |= (1 << 7); // Enable falling edge trigger on EXTI7

    // Mask the used external interrupt numbers.
    EXTI->IMR |= (1 << 7);    // Mask EXTI7

    
    
    // Set Priority for each interrupt request
    NVIC_SetPriority(EXTI9_5_IRQn, 1); // Priority level 1
    // enable EXT0 IRQ from NVIC
    NVIC_EnableIRQ(EXTI9_5_IRQn);

    //NVIC_SetPriority(TIM2_IRQn, 2); // Priority level 2
    // enable TIM2 IRQ from NVIC
    //NVIC_EnableIRQ(TIM2_IRQn);

    //NVIC_SetPriority(TIM3_IRQn, 1); // Priority level 2
    // enable TIM2 IRQ from NVIC
    //NVIC_EnableIRQ(TIM3_IRQn);

    // Enable Timer 2 module (CEN, bit0)
    //TIM2->CR1 |= (1 << 0);
    //TIM3->CR1 |= (1 << 0);

    GPIOD->ODR = (ledVal << 12);
    while(1)
    {
   
        //GPIOC->MODER |= (1 << 14); // Make pin 7 bits:14-15 an output
        //GPIOD->ODR |= (1 << 7);
        //for(int j=0; j<100; j++);
        //GPIOC->MODER &= ~((1U << 15) | (1U << 14));    // Make pin 7 bits:14-15 an input
        //while(!(EXTI->PR & (1 << 7)))
        //{

        //}
    }

    return 0;
}
